
const MAX_DELAYTOY_BITS: usize = 16;
const MAX_DELAYTOY_DELAY: usize = 1 << MAX_DELAYTOY_BITS;
const DELAYTOY_BUFFERMASK: usize = MAX_DELAYTOY_DELAY - 1;


#[derive(Clone)]
pub struct ResonatorLFO {
    cospart: f32,
    sinpart: f32,
    amppart: f32
}

impl ResonatorLFO {
    fn makenew(freq: f32) -> Self {
        let mut res = Self {
            cospart: 0.0,
            sinpart: 0.0,
            amppart: 0.0
        };
        res.set_freq(freq);
        return res;
    }
    fn get_next(&mut self) -> f32 {
        self.cospart -= self.amppart * self.sinpart;
        self.sinpart += self.amppart * self.cospart;
        return self.sinpart;
    }
    fn set_freq(&mut self, freq: f32) {
        self.amppart = freq + freq;
    }
}

#[derive(Clone)]
pub struct DelayToy {
    writeidx: usize,
    localidx: usize,
    accumulator: f32,
    feedback1: f32,
    feedback2: f32,
    feedbackflt: f32,
    buffer: Vec<f32>,
    lfo1: ResonatorLFO,
    lfo2: ResonatorLFO,
    
}

impl Default for DelayToy {
    fn default() -> Self {
        Self {
            buffer: {let mut v = Vec::new(); v.resize(MAX_DELAYTOY_DELAY, 0.0); v},
            writeidx: 0,
            localidx: 0,
            accumulator: 0.0,
            feedback1: 0.0,
            feedback2: 0.0,
            feedbackflt: 0.0,
            lfo1: ResonatorLFO::makenew(1.0 / 32777.0 * 9.4),
            lfo2: ResonatorLFO::makenew(1.3 / 32777.0 * 3.15971)
        }
    }
}

impl DelayToy {
    pub fn _clear(&mut self) {
        self.writeidx = 0;
        self.localidx = 0;
        for s in 0..MAX_DELAYTOY_DELAY {
            self.buffer[s] = 0.0;
        }
    }
    
    pub fn start(&mut self) {
        self.localidx = self.writeidx;
    }
    
    pub fn end(&mut self) {
        self.writeidx = (self.writeidx + DELAYTOY_BUFFERMASK) & DELAYTOY_BUFFERMASK;
    }
    
    /*
#define AP(len) { \
    int j = (localidx + len) & WHITERABBIT_BUFFERMASK; \
    int32_t d = Buffer[j]; \
    acc -= d >> 1; \
    Buffer[localidx] = Saturate(acc); \
    acc = (acc >> 1) + d; \
    localidx = j; \
}*/


    pub fn all_pass(&mut self, length: usize, coeff: f32) {
        let j = (self.localidx + length) & DELAYTOY_BUFFERMASK;
        let d = self.buffer[j];
        self.accumulator -= d * coeff;
        self.buffer[self.localidx] = self.saturate(self.accumulator);
        self.accumulator = (self.accumulator * coeff) + d;
        self.localidx = j;
    }
    
    pub fn linear_interpolate(&mut self, index: usize, offset: f32) -> f32
    {
        let adjustedindex = (index + MAX_DELAYTOY_DELAY - (offset.floor() as usize)) & DELAYTOY_BUFFERMASK;
        let frac = offset.fract();
        let ifrac = 1.0 - frac;
        return self.buffer[adjustedindex] * ifrac + self.buffer[(adjustedindex + 1) & DELAYTOY_BUFFERMASK] * frac;
    }
    
    pub fn all_pass_wobble(&mut self, length: usize, coeff: f32, lengthoffset: f32) {
        let j = (self.localidx + length) & DELAYTOY_BUFFERMASK;
        let d = self.linear_interpolate(j, lengthoffset);     
        self.accumulator -= d * coeff;
        self.buffer[self.localidx] = self.saturate(self.accumulator);
        self.accumulator = (self.accumulator * coeff) + d;
        self.localidx = j;
    }
    
    /*
    #define DELAY(len) { \
		int j = (localidx + len) & WHITERABBIT_BUFFERMASK; \
		Buffer[localidx] = Saturate(acc); \
		acc = Buffer[j]; \
		localidx = j; \
	}
    #define DELAY_WOBBLE(len, wobpos) { \
		int j = (localidx + len) & WHITERABBIT_BUFFERMASK; \
		Buffer[localidx] = Saturate(acc); \
		acc = LINEARINTERPRV16(Buffer, j, wobpos); \
		localidx=j; \
	}
*/
    
    pub fn delay(&mut self, length: usize) {
        let j = (self.localidx + length) & DELAYTOY_BUFFERMASK;
        self.buffer[self.localidx] = self.saturate(self.accumulator);
        self.accumulator = self.buffer[j];
        self.localidx = j;
    }
    
    
    pub fn saturate(&mut self, input: f32) -> f32 {
        if input > 1.0 {return 1.0};
        if input < -1.0 {return -1.0};
        return input;
    }
    
    pub fn test_delay(&mut self,  left: f32,  right: f32, send: f32, fade: f32)  -> (f32, f32) {
    
        let mut left_out: f32 = 0.0;
        let mut right_out: f32 = 0.0;
    
        self.start();
        self.accumulator= left + self.feedback2;
        self.all_pass(124,0.7);
        left_out += self.accumulator;
        self.all_pass(100,0.7);
        left_out += self.accumulator;
        self.all_pass(112,0.7);
        self.feedback1 = self.accumulator * fade ;
        left_out += self.accumulator;
        self.delay(2301);
        left_out += self.accumulator*0.3;
      
        self.accumulator = right + self.feedback1;
        self.all_pass(123,0.7);
        right_out += self.accumulator;
        self.all_pass(153,0.7);
        right_out += self.accumulator;
        self.all_pass(102,0.7);
        self.feedback2 = self.accumulator * fade;
        right_out += self.accumulator;
        self.delay(2350);
        right_out += self.accumulator*0.3;
      
        self.end();

        left_out = left_out * send + (1.0-send) * left;
        right_out = right_out * send + (1.0-send) * right;
        
        return (left_out, right_out);


    }

    pub fn griesinger_reverb(&mut self,  left: f32,  right: f32, send: f32, fade: f32)  -> (f32, f32) {
        let mut left_out: f32 = 0.0;
        let mut right_out: f32 = 0.0;
        self.start();
        
        self.accumulator = (left + right) * send;
        self.all_pass(142, 0.5);
        self.all_pass(379, 0.5);

        self.accumulator += (left + right) * send;
        self.all_pass(107, 0.5);
        self.all_pass(277, 0.5);
        
        let reinject = self.accumulator;
        
        let w1 = self.lfo1.get_next();
        let w2 = self.lfo2.get_next();
        
        self.accumulator += self.feedback1;
        
        self.all_pass_wobble(672, 0.5, w1);
        self.all_pass(1800, 0.5);
        self.delay(4453);
        
        left_out += self.accumulator;
        
        self.accumulator += reinject;
        self.all_pass_wobble(908, 0.5, w2);
        self.all_pass(2656, 0.5);
        self.delay(3163);
        
        right_out += self.accumulator;
        
        self.feedbackflt += (self.accumulator- self.feedbackflt )* 0.95;
        self.feedback1 = self.feedbackflt * fade;
        
        self.end();

        left_out = left_out * send + (1.0-send) * left;
        right_out = right_out * send + (1.0-send) * right;
        
        return (left_out, right_out);
        //left = left_out;
        
    }
}