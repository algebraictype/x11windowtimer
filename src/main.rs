mod arg_parser;
mod window_fetch;

use window_fetch::traverse_window_tree;

use std::error::Error;
use std::thread::sleep;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::*;
use x11rb::protocol::xproto::ConnectionExt;
use x11rb::rust_connection::RustConnection;
use x11rb::cookie::VoidCookie;
use x11rb::protocol::xproto::CreateWindowAux;
use x11rb::protocol::xproto::CreateGCAux;

use std::time::Duration;
use crate::window_fetch::get_root_window;

fn format_duration(duration: Duration) -> String {
    let total_seconds = duration.as_secs();
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}

fn draw_text_in_window(connection: &RustConnection, window: Window, gc: Gcontext, x: i16, y: i16, text: String) -> Result<(), Box<dyn Error>> {
    let text_height: i16 = 10;
    let text_length: i16 = ((text.len() * 10)/2) as i16;
    let text_as_bytes = text.as_bytes();
    let cookie: VoidCookie<RustConnection> = connection.image_text8(window, gc, (x/2 - text_length), y/2 + (text_height), text_as_bytes)?;
    if cookie.check().is_err() {
        println!("Failed to draw text");
    }
    Ok(())
}

pub fn get_window_dimensions(connection: &RustConnection, window: Window) -> Result<(i16, i16), Box<dyn Error>> {
    let geometry = connection.get_geometry(window)?.reply()?;
    Ok((geometry.width as i16, geometry.height as i16))
}

fn main() -> Result<(), Box<dyn Error>> {


    //add an option to focus the window that you want to monitor and have the program return the name of the window that is currently in focus
    //let args: Vec<String> = std::env::args().collect();
    //let window_to_monitor = validate_arguments(&args);

    //hardcoding for now
    let window_to_monitor = "Task Manager".to_string();

    //establish connection to x11 (etc)
    let (connection, screen_num) = RustConnection::connect(None)?;
    let window_id = connection.generate_id()?;
    let screen = &connection.setup().roots[screen_num];

    //traverse the user-requested window to monitor's parents to find the root
    let req_window = traverse_window_tree(&connection, screen.root, &window_to_monitor)?;
    println!("Window: {:?}", req_window);

    if(req_window == 0){
        println!("ERROR: Window not found. EXITING(1)");
        std::process::exit(0);
    }


    connection.create_window(
        x11rb::COPY_FROM_PARENT as u8,
        window_id,
        screen.root,
        0,
        0,
        200,
        200,
        0,
        WindowClass::INPUT_OUTPUT,
        0,
        &CreateWindowAux::new().event_mask(EventMask::EXPOSURE | EventMask::STRUCTURE_NOTIFY).background_pixel(screen.black_pixel),
    )?; //simply crash if error with establishing x11 window; something is seriously wrong.
    connection.map_window(window_id)?;

    let font_id = connection.generate_id()?;
    let font_handle = connection.open_font(font_id, "10x20".as_ref())?;
    if font_handle.is_err() {
        eprintln!("font couldn't be loaded.");
        std::process::exit(0);
    }
    let gc = connection.generate_id()?;

    let fonts_cookie = connection.list_fonts(100, "*".as_ref())?;

    for str in fonts_cookie.reply().unwrap().names.iter() {
        match String::from_utf8(str.name.clone()) {
            Ok(string) => println!("Font: {}", string),
            Err(e) => println!("Invalid UTF-8 sequence: {}", e),
        }
    }

    connection.create_gc(gc, window_id, &CreateGCAux::new().foreground(screen.white_pixel).background(screen.black_pixel).font(font_id))?;

    connection.flush().expect("Failed to flush connection");

    if let req_window = traverse_window_tree(&connection, screen.root, &window_to_monitor)?{
        println!("Window found");
    };

    let mut init_duration = Duration::from_secs(0);

    loop {

        let focus_reply = connection.get_input_focus()?.reply()?;
        let focus = get_root_window(&connection, focus_reply.focus)?;
        //println!("Focused window: {:?}", focus);


        sleep(Duration::from_millis(1000)); //sleep for 1 second

        if(focus == req_window){
            let mut win_timer: Duration = init_duration;
            loop {
                win_timer += Duration::from_secs(1);

                let focus_reply = connection.get_input_focus()?.reply()?;
                let focus = get_root_window(&connection, focus_reply.focus)?;
                sleep(Duration::from_secs(1)); //sleep for 1 second

                if(focus != req_window){
                    init_duration = win_timer - Duration::from_secs(1);
                    break;
                }


                sleep(Duration::from_millis(50)); //sleep for 1 second
                let time_string = format_duration(win_timer);
                let window_attr = connection.get_geometry(window_id)?.reply()?;
                draw_text_in_window(&connection, window_id, gc, window_attr.width as i16, window_attr.height as i16, time_string)?;
                connection.flush().expect("Failed to flush connection");
            }
        } else {
            //println!("Window is not in focus");
        }
        free_gc(&connection, gc).expect("Couldn't free graphics context.");
    }
}