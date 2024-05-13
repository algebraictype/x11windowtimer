use std::error::Error;
use x11rb::protocol::xproto::{Atom, AtomEnum, ConnectionExt, Window};
use x11rb::rust_connection::RustConnection;

pub fn get_window_name(connection: &RustConnection, window: Window) -> Result<String, Box<dyn Error>> {
    let atom_wm_name = connection.intern_atom(false, b"WM_NAME")?.reply()?.atom;

    let prop = connection.get_property::<Atom, u32>(false, window, atom_wm_name, AtomEnum::STRING.into(), 0, u32::MAX)?.reply()?;
    match prop.format {
        8 => {
            match String::from_utf8(prop.value) {
                Ok(name) => {
                    if name.is_empty() {
                        Err("No name found".into())
                    } else {
                        Ok(name)
                    }
                },
                Err(_) => Err("Invalid ASCII".into()),
            }
        },
        _ => Err("No name found".into()),
    }
}

pub fn get_root_window(connection: &RustConnection, window: Window) -> Result<Window, Box<dyn Error>> {
    let tree = connection.query_tree(window)?.reply()?;
    Ok(tree.parent)
}

pub fn traverse_window_tree(connection: &RustConnection, window: Window, req_win: &String) -> Result<Window, Box<dyn Error>> {
    let tree = connection.query_tree(window)?.reply()?;

    for child in tree.children {
        match get_window_name(connection, child) {
            Ok(name) =>{
                println!("Window: {:?}, Name: {:?}", child, name);
                if name == *req_win {
                    println!("MATCHED: Window: {:?}, Name: {:?}", child, name);
                    return Ok(child);
                }
            },
            Err(err) => println!("Window: {:?}, {}", child, err),
        }
        let result = traverse_window_tree(connection, child, req_win)?;
        if result != 0 {
            return Ok(result);
        }
    }

    return Ok((0));
}