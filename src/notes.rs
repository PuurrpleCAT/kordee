pub trait Note {fn f(&self) -> f32;}
pub struct A4;
pub struct F4;
pub struct C5;
impl Note for F4 {fn f(&self) -> f32{349.2282}}
impl Note for A4 {fn f(&self) -> f32{440.0000}}
impl Note for C5 {fn f(&self) -> f32{523.2511}}

