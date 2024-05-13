use std::io;
use std::io::Write;
use std::thread::sleep;
use std::time::Duration;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::rust_connection::RustConnection;
use crate::window_fetch::get_root_window;

pub fn validate_arguments(args: &Vec<String>) -> String {

    if args.len() > 2 {
        print!("Too many arguments.\nUsage: timer [-m windowtomonitor] [ ]");
        println!("To pass a window as a runtime argument use\n./timer -m \"Window Name\"\nTo supply a window name to monitor without knowing its window name, run without any arguments.");
        std::process::exit(0);
    }

    for str in args{
        match args {
            String::to_string("-m")=> {
                let window = str.split(" ")[1];
                return window;
            },
            _=> {
                eprint!("Invalid argument supplid.");
                std::process::exit(0);
            }
        };
    }

    let window_to_monitor: String = args[1].clone();
    return window_to_monitor;

}

pub fn runtime_window(connection: &RustConnection) -> bool {
    loop {
        let focus_reply = connection.get_input_focus()?.reply()?;
        let focus = get_root_window(&connection, focus_reply.focus)?;
        sleep(Duration::from_secs(1));
        println!("Focused window: {:?}", focus);
        let mut input = String::new();
        print!("Enter (y) if you would like to monitor the window that was focused previous to the window this program is running in: ");
        io::stdout().flush().unwrap(); // Make sure the prompt is immediately displayed
        io::stdin().read_line(&mut input).unwrap();
        println!("You entered: {}", input.trim());

        if(input.eq_ignore_ascii_case("y")){
            return true;
        }
    }
}
