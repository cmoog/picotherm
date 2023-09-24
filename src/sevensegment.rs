use embedded_hal::digital::{OutputPin, PinState};

pub struct SevenSeg<A, B, C, D, E, F, G> {
    on_state: PinState,
    off_state: PinState,
    segments: (A, B, C, D, E, F, G),
}

impl<A, B, C, D, E, F, G> SevenSeg<A, B, C, D, E, F, G>
where
    A: OutputPin,
    B: OutputPin,
    C: OutputPin,
    D: OutputPin,
    E: OutputPin,
    F: OutputPin,
    G: OutputPin,
{
    pub fn new(
        on_state: PinState,
        seg_a: A,
        seg_b: B,
        seg_c: C,
        seg_d: D,
        seg_e: E,
        seg_f: F,
        seg_g: G,
    ) -> Self {
        Self {
            on_state,
            off_state: if on_state == PinState::High {
                PinState::Low
            } else {
                PinState::High
            },
            segments: (seg_a, seg_b, seg_c, seg_d, seg_e, seg_f, seg_g),
        }
    }

    #[inline]
    fn new_state(&self, show: bool) -> PinState {
        if show {
            self.on_state
        } else {
            self.off_state
        }
    }

    #[allow(unused)]
    pub fn release(self) -> (A, B, C, D, E, F, G) {
        self.segments
    }

    pub fn clear(&mut self) -> Result<(), ()> {
        self.display(10)
    }

    pub fn display(&mut self, num: u8) -> Result<(), ()> {
        let spec: [bool; 7] = match num {
            0 => [true, true, true, true, true, true, false],
            1 => [false, true, true, false, false, false, false],
            2 => [true, true, false, true, true, false, true],
            3 => [true, true, true, true, false, false, true],
            4 => [false, true, true, false, false, true, true],
            5 => [true, false, true, true, false, true, true],
            6 => [true, false, true, true, true, true, true],
            7 => [true, true, true, false, false, false, false],
            8 => [true, true, true, true, true, true, true],
            9 => [true, true, true, true, false, true, true],
            _ => [false, false, false, false, false, false, false],
        };
        self.segments
            .0
            .set_state(self.new_state(spec[0]))
            .map_err(|_| ())?;
        self.segments
            .1
            .set_state(self.new_state(spec[1]))
            .map_err(|_| ())?;
        self.segments
            .2
            .set_state(self.new_state(spec[2]))
            .map_err(|_| ())?;
        self.segments
            .3
            .set_state(self.new_state(spec[3]))
            .map_err(|_| ())?;
        self.segments
            .4
            .set_state(self.new_state(spec[4]))
            .map_err(|_| ())?;
        self.segments
            .5
            .set_state(self.new_state(spec[5]))
            .map_err(|_| ())?;
        self.segments
            .6
            .set_state(self.new_state(spec[6]))
            .map_err(|_| ())?;
        Ok(())
    }
}
