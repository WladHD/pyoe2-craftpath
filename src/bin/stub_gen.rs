fn main() {
    #[cfg(feature = "python")]
    {
        let stub = match pyoe2_craftpath::py::stub_info() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to obtain stub info: {:#?}", e);
                return;
            }
        };

        match stub.generate() {
            Ok(_) => println!("Stub generated successfully"),
            Err(e) => println!("{:#?}", e),
        }
    }
}
