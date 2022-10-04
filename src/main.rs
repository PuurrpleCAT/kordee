mod init_file;
mod data;
fn main() {
    match init_file::main() {
        Ok(()) => {},
        Err(e) => {println!("{}", e)},
    }
}
