use embedded_hal::delay::DelayNs;
use embedded_hal::digital::OutputPin;

pub struct Blinky<TDelay, TOutputPin>
where
    TDelay: DelayNs,
    TOutputPin: OutputPin,
{
    delay: TDelay,
    pin: TOutputPin,
}

impl<TDelay, TOutputPin> Blinky<TDelay, TOutputPin>
where
    TDelay: DelayNs,
    TOutputPin: OutputPin,
{
    pub fn new(delay: TDelay, pin: TOutputPin) -> Self {
        Self { delay, pin }
    }

    pub fn run(mut self) -> ! {
        log::info!("Starting, blinky!");

        loop {
            self.pin.set_high().unwrap();
            // we are sleeping here to make sure the watchdog isn't triggered
            self.delay.delay_ms(1000);

            self.pin.set_low().unwrap();
            self.delay.delay_ms(1000);
        }
    }
}
