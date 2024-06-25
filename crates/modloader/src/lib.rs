use harmony_modloader_api::{Command, CommandResult, Error};

#[link(wasm_import_module = "harmony_modloader")]
extern "C" {
    fn populate_command_buffer(pointer: u32);
    fn consume_result_buffer(pointer: u32);
}

fn get_command_buffer(length: u32) -> Vec<u8> {
    let length = length as usize;
    let mut buffer = Vec::with_capacity(length);
    unsafe {
        populate_command_buffer(buffer.as_mut_ptr() as u32);
        buffer.set_len(length);
    }
    buffer
}

fn submit_result(result: CommandResult) {
    let buffer = bitcode::encode(&result);
    unsafe {
        consume_result_buffer(buffer.as_ptr() as u32);
    }
}

#[no_mangle]
pub extern "C" fn initiate(command_length: u32) {
    let command_buffer = get_command_buffer(command_length);
    let result = invoke_inner(command_buffer);
    submit_result(result);
}

fn invoke_inner(command_buffer: Vec<u8>) -> CommandResult {
    let command: Command = bitcode::decode(&command_buffer).map_err(|_| Error::DecodeCommand)?;

    match command {
        _ => unimplemented!(),
    }
}
