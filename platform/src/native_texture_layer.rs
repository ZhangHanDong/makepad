use crate::{
    makepad_live_id::LiveId,
    texture::{Texture, TextureFormat, TextureId},
    Cx,
};

#[derive(Clone, Debug, PartialEq)]
pub struct NativeTextureLayer {
    pub host_id: LiveId,
    pub texture: Texture,
    pub generation: u64,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct NativeTextureLayerFrame {
    pub host_id: LiveId,
    pub texture_id: TextureId,
    pub generation: u64,
}

impl NativeTextureLayer {
    pub fn texture(&self) -> &Texture {
        &self.texture
    }

    pub fn texture_id(&self) -> TextureId {
        self.texture.texture_id()
    }

    pub fn mark_frame_available(&mut self) -> NativeTextureLayerFrame {
        self.generation = self.generation.saturating_add(1);
        NativeTextureLayerFrame {
            host_id: self.host_id,
            texture_id: self.texture_id(),
            generation: self.generation,
        }
    }
}

pub trait NativeTextureLayerBridge {
    fn alloc_native_texture_layer(&mut self, host_id: LiveId) -> NativeTextureLayer;
}

impl NativeTextureLayerBridge for Cx {
    fn alloc_native_texture_layer(&mut self, host_id: LiveId) -> NativeTextureLayer {
        NativeTextureLayer {
            host_id,
            texture: self.textures.alloc(TextureFormat::VideoExternal),
            generation: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::Event;

    #[test]
    fn native_texture_layer_allocates_external_texture() {
        let mut cx = Cx::new(Box::new(|_cx: &mut Cx, _event: &Event| {}));
        let host_id = LiveId(42);
        let layer = cx.alloc_native_texture_layer(host_id);
        let texture_id = layer.texture_id();

        assert_eq!(layer.host_id, host_id);
        assert_eq!(layer.texture().texture_id(), texture_id);
        assert_eq!(layer.generation, 0);
        assert!(cx.textures[texture_id].format.is_video_external());
    }

    #[test]
    fn native_texture_layer_allocates_distinct_texture_per_host() {
        let mut cx = Cx::new(Box::new(|_cx: &mut Cx, _event: &Event| {}));
        let first = cx.alloc_native_texture_layer(LiveId(1));
        let second = cx.alloc_native_texture_layer(LiveId(2));

        assert_eq!(first.host_id, LiveId(1));
        assert_eq!(second.host_id, LiveId(2));
        assert_ne!(first.texture_id(), second.texture_id());
        assert!(cx.textures[first.texture_id()].format.is_video_external());
        assert!(cx.textures[second.texture_id()].format.is_video_external());
    }

    #[test]
    fn native_texture_layer_marks_frame_generations() {
        let mut cx = Cx::new(Box::new(|_cx: &mut Cx, _event: &Event| {}));
        let mut layer = cx.alloc_native_texture_layer(LiveId(7));
        let texture_id = layer.texture_id();

        let first_frame = layer.mark_frame_available();
        let second_frame = layer.mark_frame_available();

        assert_eq!(
            first_frame,
            NativeTextureLayerFrame {
                host_id: LiveId(7),
                texture_id,
                generation: 1,
            }
        );
        assert_eq!(
            second_frame,
            NativeTextureLayerFrame {
                host_id: LiveId(7),
                texture_id,
                generation: 2,
            }
        );
        assert_eq!(layer.generation, 2);
    }
}
