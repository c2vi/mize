
pub struct MizeError {
    pub message: String
}

pub fn display_error_to_user(error: MizeError){
    println!("{}", error.message);
}
