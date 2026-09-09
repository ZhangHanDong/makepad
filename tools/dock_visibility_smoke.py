#!/usr/bin/env python3
"""Drive only the supplied owned standalone Dock fixture through --remote.

Build first: cargo build --release -p makepad-example-counter --example dock_visibility
Run: python3 tools/dock_visibility_smoke.py --evidence-dir /absolute/evidence/path
"""
import argparse
import hashlib
import json
from pathlib import Path
import re
import shutil
import subprocess
import time
from urllib.parse import urlencode
from urllib.request import urlopen


def visible(row):
    return row.get("v", 1) != 0 and row["r"][2] > 0 and row["r"][3] > 0


def unique(rows, widget_id, widget_type=None):
    matches = [row for row in rows if visible(row) and row["i"] == widget_id
               and (widget_type is None or row["ty"] == widget_type)]
    if len(matches) != 1:
        raise AssertionError(f"expected one visible {widget_id}/{widget_type}, found {len(matches)}")
    return matches[0]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--binary", default="target/release/examples/dock_visibility")
    parser.add_argument("--evidence-dir", required=True)
    args = parser.parse_args()
    repo = Path(__file__).resolve().parent.parent
    binary = (repo / args.binary).resolve()
    evidence = Path(args.evidence_dir).resolve()
    evidence.mkdir(parents=True, exist_ok=True)
    if not binary.is_file():
        raise SystemExit(f"build the fixture first: {binary}")
    log_path = evidence / "fixture.log"
    port = None
    result = {"binary": str(binary), "sha256": hashlib.sha256(binary.read_bytes()).hexdigest(), "checks": []}
    with log_path.open("w") as log:
        proc = subprocess.Popen([str(binary), "--remote"], cwd=repo, stdout=log, stderr=subprocess.STDOUT)
        result["pid"] = proc.pid

        def request(path, **params):
            query = "?" + urlencode(params) if params else ""
            with urlopen(f"http://127.0.0.1:{port}{path}{query}", timeout=20) as response:
                return json.load(response)

        def snapshot(filename=None, all_rows=True):
            data = request("/snap", **({"all": 1} if all_rows else {}))
            if filename:
                (evidence / filename).write_text(json.dumps(data, indent=2) + "\n")
            return data["s"]

        def click(widget_id, widget_type=None):
            target = unique(snapshot(), widget_id, widget_type)
            x, y, width, height = target["r"]
            request("/click", x=x + width / 2, y=y + height / 2, w=target["w"], wait=1)

        def check_counts(expected):
            text = unique(snapshot(), "counts")["t"]
            actual = dict((key, int(value)) for key, value in re.findall(r"(\w+)=(\d+)", text))
            assert actual == expected, (actual, expected)

        try:
            deadline = time.monotonic() + 30
            while time.monotonic() < deadline:
                if proc.poll() is not None:
                    raise AssertionError(f"fixture exited during startup: {proc.returncode}")
                match = re.search(r"\[makepad-remote\] listening on 127\.0\.0\.1:(\d+) pid=(\d+)", log_path.read_text())
                if match:
                    assert int(match[2]) == proc.pid, "remote log PID differs from owned child"
                    port = int(match[1])
                    break
                time.sleep(0.1)
            assert port is not None, "no owned remote startup line"
            result["port"] = port
            deadline = time.monotonic() + 30
            while time.monotonic() < deadline:
                if any(row["i"] == "mode_flat" and visible(row) for row in snapshot()):
                    break
                time.sleep(0.1)
            unique(snapshot(), "mode_flat", "Button")

            tabs = ["tab_a", "tab_b", "tab_c", "tab_d", "tab_e", "tab_f"]
            for tab in tabs + ["tab_a"]:
                click(tab, "DockTab")
            rows = snapshot("flat-all.json")
            retained = [row for row in rows if row["i"] == "send" and row["r"][2] > 0 and row["r"][3] > 0]
            assert len(retained) == 6, retained
            assert len({tuple(row["r"]) for row in retained}) == 1, retained
            assert sum(visible(row) for row in retained) == 1, retained
            unique(snapshot(all_rows=False), "send", "Button")
            expected = {key: 0 for key in ["A", "B", "C", "D", "E", "F", "InnerA", "InnerB"]}
            click("send", "Button")
            expected["A"] = 1
            check_counts(expected)
            click("tab_b", "DockTab")
            click("send", "Button")
            expected["B"] = 1
            check_counts(expected)
            result["checks"].append("six overlapping retained controls; one actionable; only selected counter changes")

            click("refresh_retained", "Button")
            click("tab_f", "DockTab")
            assert unique(snapshot(), "refreshed")["t"] == "Refresh: 1"
            result["checks"].append("inactive content remains updateable through retained lookup")

            click("mode_split", "Button")
            rows = snapshot("split-all.json")
            assert sum(row["i"] == "send" and visible(row) for row in rows) == 2
            try:
                unique(rows, "send", "Button")
            except AssertionError as error:
                assert "found 2" in str(error)
            else:
                raise AssertionError("split selector silently chose one of two controls")
            check_counts(expected)
            result["checks"].append("split keeps two actionable controls; ambiguous selector sends no input")

            click("mode_nested", "Button")
            click("inner_b", "DockTab")
            click("inner_a", "DockTab")
            before = snapshot("nested-active-all.json")
            assert sum(row["ty"] == "DockTabs" and visible(row) for row in before) == 2
            click("tab_f", "DockTab")
            hidden = snapshot("nested-hidden-all.json")
            for widget_id in ["inner_a", "inner_b", "inner_a_tab", "inner_b_tab"]:
                matches = [row for row in hidden if row["i"] == widget_id]
                assert matches and all(not visible(row) for row in matches), (widget_id, matches)
            assert sum(row["ty"] == "DockTabs" and visible(row) for row in hidden) == 1
            click("nested_outer", "DockTab")
            click("send", "Button")
            expected["InnerA"] = 1
            check_counts(expected)
            assert visible(unique(snapshot(), "inner_b", "DockTab"))
            result["checks"].append("inactive nested Dock hides content and real/synthetic headers, then restores selection")
            grab = request("/g", scale=0.5)
            shutil.copy2(grab["png"], evidence / "fixture.png")
            result["status"] = "passed"
        except BaseException as error:
            result["status"] = "failed"
            result["error"] = repr(error)
            raise
        finally:
            try:
                if proc.poll() is None and port is not None:
                    try:
                        request("/gq", scale=0.25)
                    except Exception as error:
                        result["quit_error"] = repr(error)
                try:
                    proc.wait(timeout=10)
                except subprocess.TimeoutExpired:
                    result["forced_cleanup"] = True
                    proc.terminate()
                    try:
                        proc.wait(timeout=10)
                    except subprocess.TimeoutExpired:
                        proc.kill()
                        proc.wait(timeout=10)
                result["exit_code"] = proc.returncode
                if result.get("status") == "passed" and (proc.returncode != 0 or result.get("forced_cleanup")):
                    result["status"] = "failed"
                    result["error"] = "fixture did not exit gracefully after /gq"
            finally:
                (evidence / "result.json").write_text(json.dumps(result, indent=2) + "\n")
    assert result["status"] == "passed" and result["exit_code"] == 0, result
    print(json.dumps(result, indent=2))


if __name__ == "__main__":
    main()
