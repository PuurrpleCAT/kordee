mod init_file;
mod data;
mod notes;
mod chord;
fn main() {
    match init_file::main() {
        Ok(()) => {},
        Err(e) => {println!("{}", e)},
    }
}
