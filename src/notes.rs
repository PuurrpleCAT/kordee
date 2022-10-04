pub trait Note {}
pub struct A4 {f: f32}
pub struct F4 {f: f32}
pub struct C5 {f: f32}
impl F4 {fn f() -> f32 {349.2282}}
impl A4 {fn f() -> f32 {440.0000}}
impl C5 {fn f() -> f32 {523.2511}}
impl Note for A4 {}
impl Note for F4 {}
impl Note for C5 {}

