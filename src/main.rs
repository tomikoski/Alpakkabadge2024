#![no_std]
#![no_main]
#![warn(clippy::all, clippy::pedantic, clippy::nursery)]

use panic_halt as _;

mod bsp;
use bsp::prelude::*;

// Traits
use embedded_hal::digital::v2::OutputPin; // for pin.toggle()
use embedded_hal::PwmPin;
use hal::clocks::Clock; // for system_clock.freq()

use palette::{IntoColor, Srgb, Hsv};
use embedded_hal::adc::OneShot;
use rp2040_hal::{adc::Adc, pac};

// Fahrenheit -> Celsius
fn convert_to_celsius(raw_temp: u16) -> u16 {
    // According to chapter 4.9.5. Temperature Sensor in RP2040 datasheet
    let temp = 27.0 - (raw_temp as f32 * 3.3 / 4096.0 - 0.706) / 0.001721;
    let sign = if temp < 0.0 { -1.0 } else { 1.0 };
    let rounded_temp_x10: i16 = ((temp * 10.0) + 0.5 * sign) as i16;
    (rounded_temp_x10 as u16) / 10
}

#[entry]
fn main() -> ! {
    let mut pac = pac::Peripherals::take().unwrap();
    let core = pac::CorePeripherals::take().unwrap();

    let mut watchdog = hal::Watchdog::new(pac.WATCHDOG);

    let clocks = hal::clocks::init_clocks_and_plls(
        bsp::XOSC_CRYSTAL_FREQ,
        pac.XOSC,
        pac.CLOCKS,
        pac.PLL_SYS,
        pac.PLL_USB,
        &mut pac.RESETS,
        &mut watchdog,
    )
    .ok()
    .unwrap();

    let mut delay = cortex_m::delay::Delay::new(core.SYST, clocks.system_clock.freq().to_Hz());
    let mut pwm_slices = hal::pwm::Slices::new(pac.PWM, &mut pac.RESETS);

    let sio = hal::Sio::new(pac.SIO);
    let pins = bsp::Pins::new(
        pac.IO_BANK0,
        pac.PADS_BANK0,
        sio.gpio_bank0,
        &mut pac.RESETS,
    );

    let lr: bsp::PWM7 = pins.pwm7.into_mode();
    let lb: bsp::PWM8 = pins.pwm8.into_mode();
    let lg: bsp::PWM9 = pins.pwm9.into_mode();
    let rr: bsp::PWM10 = pins.pwm10.into_mode();
    let rb: bsp::PWM11 = pins.pwm11.into_mode();
    let rg: bsp::PWM12 = pins.pwm12.into_mode();
    let hr: bsp::PWM13 = pins.pwm13.into_mode();
    let hb: bsp::PWM14 = pins.pwm14.into_mode();
    let hg: bsp::PWM15 = pins.pwm15.into_mode();

    let pwm3 = &mut pwm_slices.pwm3;
    pwm3.set_ph_correct();
    pwm3.enable();
    let pwm4 = &mut pwm_slices.pwm4;
    pwm4.set_ph_correct();
    pwm4.enable();
    let pwm5 = &mut pwm_slices.pwm5;
    pwm5.set_ph_correct();
    pwm5.enable();
    let pwm6 = &mut pwm_slices.pwm6;
    pwm6.set_ph_correct();
    pwm6.enable();
    let pwm7 = &mut pwm_slices.pwm7;
    pwm7.set_ph_correct();
    pwm7.enable();

    let plr = &mut pwm3.channel_b;
    plr.set_inverted();
    let plb = &mut pwm4.channel_a;
    plb.set_inverted();
    let plg = &mut pwm4.channel_b;
    plg.set_inverted();
    let prr = &mut pwm5.channel_a;
    prr.set_inverted();
    let prb = &mut pwm5.channel_b;
    prb.set_inverted();
    let prg = &mut pwm6.channel_a;
    prg.set_inverted();
    let phr = &mut pwm6.channel_b;
    phr.set_inverted();
    let phb = &mut pwm7.channel_a;
    phb.set_inverted();
    let phg = &mut pwm7.channel_b;
    phg.set_inverted();

    plr.output_to(lr);
    plr.set_duty(0);
    plb.output_to(lb);
    plb.set_duty(0);
    plg.output_to(lg);
    plg.set_duty(0);
    prr.output_to(rr);
    prr.set_duty(0);
    prb.output_to(rb);
    prb.set_duty(0);
    prg.output_to(rg);
    prg.set_duty(0);
    phr.output_to(hr);
    phr.set_duty(0);
    phb.output_to(hb);
    phb.set_duty(0);
    phg.output_to(hg);
    phg.set_duty(0);

    let mut led: bsp::Led = pins.led.into_mode();
    led.set_low().unwrap();

    // enable ADC with TempSense: https://docs.rs/rp2040-hal/0.7.0/rp2040_hal/adc/index.html
    let mut adc = Adc::new(pac.ADC, &mut pac.RESETS);
    let mut temperature_sensor = adc.enable_temp_sensor();

    let mut heart1 = 0;
    let mut heart2 = 0;
    let mut feeling_cold: bool = false;
    pub const MY_ALPACCA_FEELS_COLD_WHEN_CELSIUS_HITS_UNDER: u16 = 23;

    loop {
        for time in 0u16..65500 {
            let eyes: Srgb = Hsv::new(f32::from(time)/65500.0*360.0*40.0, 1.0, 1.0).into_color();
            let eyes_components = eyes.into_components();
            let eye_r = (eyes_components.0 * 20000.0) as u16;
            let eye_g = (eyes_components.1 * 20000.0) as u16;
            let eye_b = (eyes_components.2 * 65535.0) as u16;            
            plr.set_duty(eye_r);
            plg.set_duty(eye_g);
            plb.set_duty(eye_b);
            prr.set_duty(eye_r);
            prg.set_duty(eye_g);
            prb.set_duty(eye_b);

            if time.wrapping_add(20) % 100 == 0 {
                heart1 = 0xffff;
            }

            // Change of <3
            if time % 100 == 0 {
                let temperature_adc_counts: u16 = adc.read(&mut temperature_sensor).unwrap();
                let temperature = convert_to_celsius(temperature_adc_counts);
                match temperature {
                    0 .. MY_ALPACCA_FEELS_COLD_WHEN_CELSIUS_HITS_UNDER => feeling_cold = true,
                    MY_ALPACCA_FEELS_COLD_WHEN_CELSIUS_HITS_UNDER..=u16::MAX => feeling_cold = false,
                }                
                heart1 = 0xffff;
                heart2 = 0x1000;
            }

            // Give either BLUE or RED <3
            match feeling_cold {
                true => {
                            // Blue <3
                            phr.set_duty(heart2); //heart red
                            //phg.set_duty(0); //heart green
                            phb.set_duty(heart1); //heart blue
                }
                false => {
                            // Red <3
                            phr.set_duty(heart1); //heart red
                            //phg.set_duty(heart1); //heart green
                            phb.set_duty(heart2); //heart blue
                }
            }

            heart1 = match heart1.checked_sub(1000) {
                Some(n) => n,
                None => 0,
            };

            heart2 = match heart2.checked_sub(1000) {
                Some(n) => n,
                None => 0,
            };

            delay.delay_ms(100);
        }
    }
}
