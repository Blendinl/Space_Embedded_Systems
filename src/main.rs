use std::error::Error;
use std::io;
use std::thread;
use std::time::Duration;
use rppal::gpio::Gpio;
use rppal::system::DeviceInfo;
use uln2003::{Direction, Stepper};

fn main() -> Result<(), Box<dyn Error>> {
    println!("Stepper Motor Control Program");
    println!("------------------------------");

    // Check if the program is running on Raspberry Pi
    if DeviceInfo::new()?.model() != "Raspberry Pi 3 Model B Rev 1.2" {
        eprintln!("This program is designed to run on Raspberry Pi 3 only.");
        return Ok(());
    }

    // Initialize GPIO
    let gpio = Gpio::new()?;

    // Initialize the main stepper motor (Clockwise)
    let mut stepper_main = Stepper::new(17, 27, 22, 10)?;

    // Initialize the backup stepper motor (Counterclockwise)
    let mut stepper_backup = Stepper::new(5, 6, 13, 19)?;

    loop {
        println!("Enter the number of degrees and motor selection (e.g., 90, 1 for motor 1 or 180, 2 for motor 2), or '0' to exit:");

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        let (degrees, motor_select): (i32, u8) = match input.trim().split(',').map(|s| s.trim().parse()).collect::<Result<Vec<_>, _>>() {
            Ok(values) if values.len() == 2 => (values[0], values[1]),
            _ => {
                println!("Invalid input. Please enter degrees followed by motor selection (e.g., 90, 1 or 180, 2).");
                continue;
            }
        };

        if degrees == 0 {
            println!("Exiting program.");
            break;
        }

        let mut target_degrees = degrees;
        let mut motor = match motor_select {
            1 => &mut stepper_main,
            2 => &mut stepper_backup,
            _ => {
                println!("Invalid motor selection. Please choose 1 or 2.");
                continue;
            }
        };

        // Ensure that degrees is positive for stepper_main and negative for stepper_backup
        if (motor_select == 1 && degrees < 0) || (motor_select == 2 && degrees > 0) {
            target_degrees = (target_degrees + 360) % 360; // Map to [0, 360) range
        }

        // Calculate the current position of the motor
        let current_position = motor.current_position();

        // Calculate the difference in degrees between the current position and the target position
        let mut degrees_to_move = (target_degrees - current_position + 360) % 360;

        // If the target position is in the opposite direction, complete a full rotation
        if (motor_select == 1 && degrees_to_move < 0) || (motor_select == 2 && degrees_to_move > 0) {
            degrees_to_move += 360;
        }

        // Rotate the motor to the target position
        motor.rotate_degrees(if motor_select == 1 { Direction::Clockwise } else { Direction::Counterclockwise }, degrees_to_move as u32);
        thread::sleep(Duration::from_millis(500)); // Wait for motor movement to complete
    }

    Ok(())
}
