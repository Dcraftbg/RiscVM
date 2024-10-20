use core::fmt;

pub struct Off32(pub i32);
impl fmt::Display for Off32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0 >= 0 {
            write!(f, "+{}",self.0)
        } else {
            write!(f, "{}",self.0)
        }
    }
}
