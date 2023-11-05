use crt_like_js_renderer::start;

fn main(){
    pollster::block_on(start());
}