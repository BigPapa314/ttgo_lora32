#![no_std]

use bit_field::BitField;
use embedded_hal::delay::DelayNs;
use embedded_hal::digital;
use embedded_hal::digital::OutputPin;
use embedded_hal::spi;
use embedded_hal::spi::SpiDevice;

mod register;
use self::register::IRQ;
use self::register::PaConfig;
use self::register::Register;

/// Provides the necessary SPI mode configuration for the radio
pub const MODE: spi::Mode = spi::MODE_3;

/// Provides high-level access to Semtech SX1276/77/78/79 based boards
pub struct LoRa<SPI_DEVICE, RESET_PIN, DELAY_NS> {
    spi: SPI_DEVICE,
    reset_pin: RESET_PIN,
    delay: DELAY_NS,
    frequency_mhz: u16,
    explicit_header: bool,
    mode: RadioMode,
}

#[derive(Debug)]
pub enum Error<SPI_DEVICE_ERROR, RESET_PIN_ERROR> {
    Uninformative,
    VersionMismatch(u8),
    Reset(RESET_PIN_ERROR),
    SPI(SPI_DEVICE_ERROR),
    Transmitting,
}

use crate::register::{FskDataModulationShaping, FskRampUpRamDown};
use Error::*;

const VERSION_CHECK: u8 = 0x12;

impl<SPI_DEVICE, RESET_PIN, DELAY_NS> LoRa<SPI_DEVICE, RESET_PIN, DELAY_NS>
where
    SPI_DEVICE: SpiDevice,
    RESET_PIN: OutputPin,
    DELAY_NS: DelayNs,
{
    /// Builds and returns a new instance of the radio. Only one instance of the radio should exist at a time.
    /// This also preforms a hardware reset of the module and then puts it in standby.
    pub fn new(
        spi: SPI_DEVICE,
        reset_pin: RESET_PIN,
        delay: DELAY_NS,
        frequency_mhz: u16,
    ) -> Result<Self, Error<SPI_DEVICE::Error, RESET_PIN::Error>> {
        let mut sx127x = LoRa {
            spi,
            reset_pin,
            delay,
            frequency_mhz: 0,
            explicit_header: true,
            mode: RadioMode::Sleep,
        };

        sx127x.reset()?;

        let version = sx127x.read_register(Register::RegVersion.addr())?;
        if version != VERSION_CHECK {
            return Err(Error::VersionMismatch(version));
        }

        sx127x.set_mode(RadioMode::Sleep)?;
        sx127x.set_frequency(frequency_mhz)?;
        sx127x.write_register(Register::RegFifoTxBaseAddr.addr(), 0)?;
        sx127x.write_register(Register::RegFifoRxBaseAddr.addr(), 0)?;
        let lna = sx127x.read_register(Register::RegLna.addr())?;
        sx127x.write_register(Register::RegLna.addr(), lna | 0x03)?;
        sx127x.write_register(Register::RegModemConfig3.addr(), 0x04)?;
        sx127x.set_mode(RadioMode::Stdby)?;
        Ok(sx127x)
    }

    pub fn reset(&mut self) -> Result<(), Error<SPI_DEVICE::Error, RESET_PIN::Error>> {
        self.reset.set_low().map_err(Reset)?;
        self.delay.delay_ms(10);
        self.reset.set_high().map_err(Reset)?;
        self.delay.delay_ms(10);
        Ok(())
    }

    /// Lets owner of the driver struct to reconfigure the radio.  Takes care of resetting the
    /// chip, putting it into a sleep mode and pulling CS high - thought he caller has to put if
    /// back to some of the active modes himself
    pub fn configure<F>(
        &mut self,
        modifier: F,
        delay: &mut dyn DelayMs<u8>,
    ) -> Result<(), Error<E, CS::Error, RESET::Error>>
    where
        F: FnOnce(&mut Self) -> Result<(), Error<E, CS::Error, RESET::Error>>,
    {
        self.reset(delay)?;
        self.set_mode(RadioMode::Sleep)?;
        modifier(self)?;
        self.cs.set_high().map_err(CS)?;
        Ok(())
    }

    /// Transmits up to 255 bytes of data. To avoid the use of an allocator, this takes a fixed 255 u8
    /// array and a payload size and returns the number of bytes sent if successful.
    pub fn transmit_payload_busy(
        &mut self,
        buffer: [u8; 255],
        payload_size: usize,
    ) -> Result<usize, Error<E, CS::Error, RESET::Error>> {
        if self.transmitting()? {
            Err(Transmitting)
        } else {
            self.set_mode(RadioMode::Stdby)?;
            if self.explicit_header {
                self.set_explicit_header_mode()?;
            } else {
                self.set_implicit_header_mode()?;
            }

            self.write_register(Register::RegIrqFlags.addr(), 0)?;
            self.write_register(Register::RegFifoAddrPtr.addr(), 0)?;
            self.write_register(Register::RegPayloadLength.addr(), 0)?;
            for byte in buffer.iter().take(payload_size) {
                self.write_register(Register::RegFifo.addr(), *byte)?;
            }
            self.write_register(Register::RegPayloadLength.addr(), payload_size as u8)?;
            self.set_mode(RadioMode::Tx)?;
            while self.transmitting()? {}
            Ok(payload_size)
        }
    }

    pub fn set_dio0_tx_done(&mut self) -> Result<(), Error<E, CS::Error, RESET::Error>> {
        self.write_register(Register::RegDioMapping1.addr(), 0b01_00_00_00)
    }

    pub fn transmit_payload(
        &mut self,
        payload: &[u8],
    ) -> Result<(), Error<E, CS::Error, RESET::Error>> {
        if self.transmitting()? {
            Err(Transmitting)
        } else {
            self.set_mode(RadioMode::Stdby)?;
            if self.explicit_header {
                self.set_explicit_header_mode()?;
            } else {
                self.set_implicit_header_mode()?;
            }

            self.write_register(Register::RegIrqFlags.addr(), 0)?;
            self.write_register(Register::RegFifoAddrPtr.addr(), 0)?;
            self.write_register(Register::RegPayloadLength.addr(), 0)?;
            for &byte in payload.iter().take(255) {
                self.write_register(Register::RegFifo.addr(), byte)?;
            }
            self.write_register(
                Register::RegPayloadLength.addr(),
                payload.len().min(255) as u8,
            )?;
            self.set_mode(RadioMode::Tx)?;
            Ok(())
        }
    }

    /// Blocks the current thread, returning the size of a packet if one is received or an error is the
    /// task timed out. The timeout can be supplied with None to make it poll indefinitely or
    /// with `Some(timeout_in_mill_seconds)`
    pub fn poll_irq(
        &mut self,
        timeout_ms: Option<i32>,
        delay: &mut dyn DelayMs<u8>,
    ) -> Result<usize, Error<E, CS::Error, RESET::Error>> {
        self.set_mode(RadioMode::RxContinuous)?;
        match timeout_ms {
            Some(value) => {
                let mut count = 0;
                let packet_ready = loop {
                    let packet_ready = self.read_register(Register::RegIrqFlags.addr())?.get_bit(6);
                    if count >= value || packet_ready {
                        break packet_ready;
                    }
                    count += 1;
                    delay.delay_ms(1);
                };
                if packet_ready {
                    self.clear_irq()?;
                    Ok(self.read_register(Register::RegRxNbBytes.addr())? as usize)
                } else {
                    Err(Uninformative)
                }
            }
            None => {
                while !self.read_register(Register::RegIrqFlags.addr())?.get_bit(6) {
                    delay.delay_ms(100);
                }
                self.clear_irq()?;
                Ok(self.read_register(Register::RegRxNbBytes.addr())? as usize)
            }
        }
    }

    /// Returns the contents of the fifo as a fixed 255 u8 array. This should only be called if there is a
    /// new packet ready to be read.
    pub fn read_packet(&mut self) -> Result<[u8; 255], Error<E, CS::Error, RESET::Error>> {
        let mut buffer = [0 as u8; 255];
        self.clear_irq()?;
        let size = self.get_ready_packet_size()?;
        let fifo_addr = self.read_register(Register::RegFifoRxCurrentAddr.addr())?;
        self.write_register(Register::RegFifoAddrPtr.addr(), fifo_addr)?;
        for i in 0..size {
            let byte = self.read_register(Register::RegFifo.addr())?;
            buffer[i as usize] = byte;
        }
        self.write_register(Register::RegFifoAddrPtr.addr(), 0)?;
        Ok(buffer)
    }

    /// Returns size of a packet read into FIFO. This should only be calle if there is a new packet
    /// ready to be read.
    pub fn get_ready_packet_size(&mut self) -> Result<u8, Error<E, CS::Error, RESET::Error>> {
        self.read_register(Register::RegRxNbBytes.addr())
    }

    /// Returns true if the radio is currently transmitting a packet.
    pub fn transmitting(&mut self) -> Result<bool, Error<E, CS::Error, RESET::Error>> {
        let op_mode = self.read_register(Register::RegOpMode.addr())?;
        if (op_mode & RadioMode::Tx.addr()) == RadioMode::Tx.addr()
            || (op_mode & RadioMode::FsTx.addr()) == RadioMode::FsTx.addr()
        {
            Ok(true)
        } else {
            if (self.read_register(Register::RegIrqFlags.addr())? & IRQ::IrqTxDoneMask.addr()) == 1
            {
                self.write_register(Register::RegIrqFlags.addr(), IRQ::IrqTxDoneMask.addr())?;
            }
            Ok(false)
        }
    }

    /// Clears the radio's IRQ registers.
    pub fn clear_irq(&mut self) -> Result<(), Error<E, CS::Error, RESET::Error>> {
        let irq_flags = self.read_register(Register::RegIrqFlags.addr())?;
        self.write_register(Register::RegIrqFlags.addr(), irq_flags)
    }

    /// Sets the transmit power and pin. Levels can range from 0-14 when the output
    /// pin = 0(RFO), and form 0-20 when output pin = 1(PaBoost). Power is in dB.
    /// Default value is `17`.
    pub fn set_tx_power(
        &mut self,
        mut level: i32,
        output_pin: u8,
    ) -> Result<(), Error<E, CS::Error, RESET::Error>> {
        if PaConfig::PaOutputRfoPin.addr() == output_pin {
            // RFO
            if level < 0 {
                level = 0;
            } else if level > 14 {
                level = 14;
            }
            self.write_register(Register::RegPaConfig.addr(), (0x70 | level) as u8)
        } else {
            // PA BOOST
            if level > 17 {
                if level > 20 {
                    level = 20;
                }
                // subtract 3 from level, so 18 - 20 maps to 15 - 17
                level -= 3;

                // High Power +20 dBm Operation (Semtech SX1276/77/78/79 5.4.3.)
                self.write_register(Register::RegPaDac.addr(), 0x87)?;
                self.set_ocp(140)?;
            } else {
                if level < 2 {
                    level = 2;
                }
                //Default value PA_HF/LF or +17dBm
                self.write_register(Register::RegPaDac.addr(), 0x84)?;
                self.set_ocp(100)?;
            }
            level -= 2;
            self.write_register(
                Register::RegPaConfig.addr(),
                PaConfig::PaBoost.addr() | level as u8,
            )
        }
    }

    /// Sets the over current protection on the radio(mA).
    pub fn set_ocp(&mut self, ma: u8) -> Result<(), Error<E, CS::Error, RESET::Error>> {
        let mut ocp_trim: u8 = 27;

        if ma <= 120 {
            ocp_trim = (ma - 45) / 5;
        } else if ma <= 240 {
            ocp_trim = (ma + 30) / 10;
        }
        self.write_register(Register::RegOcp.addr(), 0x20 | (0x1F & ocp_trim))
    }

    /// Sets the state of the radio. Default mode after initiation is `Standby`.
    pub fn set_mode(&mut self, mode: RadioMode) -> Result<(), Error<E, CS::Error, RESET::Error>> {
        if self.explicit_header {
            self.set_explicit_header_mode()?;
        } else {
            self.set_implicit_header_mode()?;
        }
        self.write_register(
            Register::RegOpMode.addr(),
            RadioMode::LongRangeMode.addr() | mode.addr(),
        )?;

        self.mode = mode;
        Ok(())
    }

    /// Sets the frequency of the radio. Values are in megahertz.
    /// I.E. 915 MHz must be used for North America. Check regulation for your area.
    pub fn set_frequency(&mut self, freq: i64) -> Result<(), Error<E, CS::Error, RESET::Error>> {
        self.frequency = freq;
        // calculate register values
        let base = 1;
        let frf = (freq * (base << 19)) / 32;
        // write registers
        self.write_register(
            Register::RegFrfMsb.addr(),
            ((frf & 0x00FF_0000) >> 16) as u8,
        )?;
        self.write_register(Register::RegFrfMid.addr(), ((frf & 0x0000_FF00) >> 8) as u8)?;
        self.write_register(Register::RegFrfLsb.addr(), (frf & 0x0000_00FF) as u8)
    }

    /// Sets the radio to use an explicit header. Default state is `ON`.
    fn set_explicit_header_mode(&mut self) -> Result<(), Error<E, CS::Error, RESET::Error>> {
        let reg_modem_config_1 = self.read_register(Register::RegModemConfig1.addr())?;
        self.write_register(Register::RegModemConfig1.addr(), reg_modem_config_1 & 0xfe)?;
        self.explicit_header = true;
        Ok(())
    }

    /// Sets the radio to use an implicit header. Default state is `OFF`.
    fn set_implicit_header_mode(&mut self) -> Result<(), Error<E, CS::Error, RESET::Error>> {
        let reg_modem_config_1 = self.read_register(Register::RegModemConfig1.addr())?;
        self.write_register(Register::RegModemConfig1.addr(), reg_modem_config_1 & 0x01)?;
        self.explicit_header = false;
        Ok(())
    }

    /// Sets the spreading factor of the radio. Supported values are between 6 and 12.
    /// If a spreading factor of 6 is set, implicit header mode must be used to transmit
    /// and receive packets. Default value is `7`.
    pub fn set_spreading_factor(
        &mut self,
        mut sf: u8,
    ) -> Result<(), Error<E, CS::Error, RESET::Error>> {
        if sf < 6 {
            sf = 6;
        } else if sf > 12 {
            sf = 12;
        }

        if sf == 6 {
            self.write_register(Register::RegDetectionOptimize.addr(), 0xc5)?;
            self.write_register(Register::RegDetectionThreshold.addr(), 0x0c)?;
        } else {
            self.write_register(Register::RegDetectionOptimize.addr(), 0xc3)?;
            self.write_register(Register::RegDetectionThreshold.addr(), 0x0a)?;
        }
        let modem_config_2 = self.read_register(Register::RegModemConfig2.addr())?;
        self.write_register(
            Register::RegModemConfig2.addr(),
            (modem_config_2 & 0x0f) | ((sf << 4) & 0xf0),
        )?;
        self.set_ldo_flag()?;
        Ok(())
    }

    /// Sets the signal bandwidth of the radio. Supported values are: `7800 Hz`, `10400 Hz`,
    /// `15600 Hz`, `20800 Hz`, `31250 Hz`,`41700 Hz` ,`62500 Hz`,`125000 Hz` and `250000 Hz`
    /// Default value is `125000 Hz`
    /// See p. 4 of SX1276_77_8_ErrataNote_1.1_STD.pdf for Errata implemetation
    pub fn set_signal_bandwidth(
        &mut self,
        sbw: i64,
    ) -> Result<(), Error<E, CS::Error, RESET::Error>> {
        let bw: i64 = match sbw {
            7_800 => 0,
            10_400 => 1,
            15_600 => 2,
            20_800 => 3,
            31_250 => 4,
            41_700 => 5,
            62_500 => 6,
            125_000 => 7,
            250_000 => 8,
            _ => 9,
        };

        if bw == 9 {
            if self.frequency < 525 {
                self.write_register(Register::RegHighBWOptimize1.addr(), 0x02)?;
                self.write_register(Register::RegHighBWOptimize2.addr(), 0x7f)?;
            } else {
                self.write_register(Register::RegHighBWOptimize1.addr(), 0x02)?;
                self.write_register(Register::RegHighBWOptimize2.addr(), 0x64)?;
            }
        } else {
            self.write_register(Register::RegHighBWOptimize1.addr(), 0x03)?;
            self.write_register(Register::RegHighBWOptimize2.addr(), 0x65)?;
        }

        let modem_config_1 = self.read_register(Register::RegModemConfig1.addr())?;
        self.write_register(
            Register::RegModemConfig1.addr(),
            (modem_config_1 & 0x0f) | ((bw << 4) as u8),
        )?;
        self.set_ldo_flag()?;
        Ok(())
    }

    /// Sets the coding rate of the radio with the numerator fixed at 4. Supported values
    /// are between `5` and `8`, these correspond to coding rates of `4/5` and `4/8`.
    /// Default value is `5`.
    pub fn set_coding_rate_4(
        &mut self,
        mut denominator: u8,
    ) -> Result<(), Error<E, CS::Error, RESET::Error>> {
        if denominator < 5 {
            denominator = 5;
        } else if denominator > 8 {
            denominator = 8;
        }
        let cr = denominator - 4;
        let modem_config_1 = self.read_register(Register::RegModemConfig1.addr())?;
        self.write_register(
            Register::RegModemConfig1.addr(),
            (modem_config_1 & 0xf1) | (cr << 1),
        )
    }

    /// Sets the preamble length of the radio. Values are between 6 and 65535.
    /// Default value is `8`.
    pub fn set_preamble_length(
        &mut self,
        length: i64,
    ) -> Result<(), Error<E, CS::Error, RESET::Error>> {
        self.write_register(Register::RegPreambleMsb.addr(), (length >> 8) as u8)?;
        self.write_register(Register::RegPreambleLsb.addr(), length as u8)
    }

    /// Enables are disables the radio's CRC check. Default value is `false`.
    pub fn set_crc(&mut self, value: bool) -> Result<(), Error<E, CS::Error, RESET::Error>> {
        let modem_config_2 = self.read_register(Register::RegModemConfig2.addr())?;
        if value {
            self.write_register(Register::RegModemConfig2.addr(), modem_config_2 | 0x04)
        } else {
            self.write_register(Register::RegModemConfig2.addr(), modem_config_2 & 0xfb)
        }
    }

    /// Inverts the radio's IQ signals. Default value is `false`.
    pub fn set_invert_iq(&mut self, value: bool) -> Result<(), Error<E, CS::Error, RESET::Error>> {
        if value {
            self.write_register(Register::RegInvertiq.addr(), 0x66)?;
            self.write_register(Register::RegInvertiq2.addr(), 0x19)
        } else {
            self.write_register(Register::RegInvertiq.addr(), 0x27)?;
            self.write_register(Register::RegInvertiq2.addr(), 0x1d)
        }
    }

    /// Returns the spreading factor of the radio.
    pub fn get_spreading_factor(&mut self) -> Result<u8, Error<E, CS::Error, RESET::Error>> {
        Ok(self.read_register(Register::RegModemConfig2.addr())? >> 4)
    }

    /// Returns the signal bandwidth of the radio.
    pub fn get_signal_bandwidth(&mut self) -> Result<i64, Error<E, CS::Error, RESET::Error>> {
        let bw = self.read_register(Register::RegModemConfig1.addr())? >> 4;
        let bw = match bw {
            0 => 7_800,
            1 => 10_400,
            2 => 15_600,
            3 => 20_800,
            4 => 31_250,
            5 => 41_700,
            6 => 62_500,
            7 => 125_000,
            8 => 250_000,
            9 => 500_000,
            _ => -1,
        };
        Ok(bw)
    }

    /// Returns the RSSI of the last received packet.
    pub fn get_packet_rssi(&mut self) -> Result<i32, Error<E, CS::Error, RESET::Error>> {
        Ok(i32::from(self.read_register(Register::RegPktRssiValue.addr())?) - 157)
    }

    /// Returns the signal to noise radio of the the last received packet.
    pub fn get_packet_snr(&mut self) -> Result<f64, Error<E, CS::Error, RESET::Error>> {
        Ok(f64::from(
            self.read_register(Register::RegPktSnrValue.addr())?,
        ))
    }

    /// Returns the frequency error of the last received packet in Hz.
    pub fn get_packet_frequency_error(&mut self) -> Result<i64, Error<E, CS::Error, RESET::Error>> {
        let mut freq_error: i32 = 0;
        freq_error = i32::from(self.read_register(Register::RegFreqErrorMsb.addr())? & 0x7);
        freq_error <<= 8i64;
        freq_error += i32::from(self.read_register(Register::RegFreqErrorMid.addr())?);
        freq_error <<= 8i64;
        freq_error += i32::from(self.read_register(Register::RegFreqErrorLsb.addr())?);

        let f_xtal = 32_000_000; // FXOSC: crystal oscillator (XTAL) frequency (2.5. Chip Specification, p. 14)
        let f_error = ((f64::from(freq_error) * (1i64 << 24) as f64) / f64::from(f_xtal))
            * (self.get_signal_bandwidth()? as f64 / 500_000.0f64); // p. 37
        Ok(f_error as i64)
    }

    fn set_ldo_flag(&mut self) -> Result<(), Error<E, CS::Error, RESET::Error>> {
        let sw = self.get_signal_bandwidth()?;
        // Section 4.1.1.5
        let symbol_duration = 1000 / (sw / ((1 as i64) << self.get_spreading_factor()?));

        // Section 4.1.1.6
        let ldo_on = symbol_duration > 16;

        let mut config_3 = self.read_register(Register::RegModemConfig3.addr())?;
        config_3.set_bit(3, ldo_on);
        self.write_register(Register::RegModemConfig3.addr(), config_3)
    }

    fn read_register(&mut self, reg: u8) -> Result<u8, Error<E, CS::Error, RESET::Error>> {
        self.cs.set_low().map_err(CS)?;

        let mut buffer = [reg & 0x7f, 0];
        let transfer = self.spi.transfer(&mut buffer).map_err(SPI)?;
        self.cs.set_high().map_err(CS)?;
        Ok(transfer[1])
    }

    fn write_register(
        &mut self,
        reg: u8,
        byte: u8,
    ) -> Result<(), Error<E, CS::Error, RESET::Error>> {
        self.cs.set_low().map_err(CS)?;

        let buffer = [reg | 0x80, byte];
        self.spi.write(&buffer).map_err(SPI)?;
        self.cs.set_high().map_err(CS)?;
        Ok(())
    }

    pub fn put_in_fsk_mode(&mut self) -> Result<(), Error<E, CS::Error, RESET::Error>> {
        // Put in FSK mode
        let mut op_mode: u8 = 0x0;
        op_mode
            .set_bit(7, false) // FSK mode
            .set_bits(5..6, 0x00) // FSK modulation
            .set_bit(3, false) //Low freq registers
            .set_bits(0..2, 0b011); // Mode

        self.write_register(Register::RegOpMode as u8, op_mode)
    }

    pub fn set_fsk_pa_ramp(
        &mut self,
        modulation_shaping: FskDataModulationShaping,
        ramp: FskRampUpRamDown,
    ) -> Result<(), Error<E, CS::Error, RESET::Error>> {
        let mut pa_ramp: u8 = 0x0;
        pa_ramp
            .set_bits(5..6, modulation_shaping as u8)
            .set_bits(0..3, ramp as u8);

        self.write_register(Register::RegPaRamp as u8, pa_ramp)
    }
}
/// Modes of the radio and their corresponding register values.
#[derive(Clone, Copy)]
pub enum RadioMode {
    LongRangeMode = 0x80,
    Sleep = 0x00,
    Stdby = 0x01,
    FsTx = 0x02,
    Tx = 0x03,
    RxContinuous = 0x05,
    RxSingle = 0x06,
}

impl RadioMode {
    /// Returns the address of the mode.
    pub fn addr(self) -> u8 {
        self as u8
    }
}
