extern crate sdl2;
extern crate time;

use sdl2::pixels::PixelFormatEnum;
use sdl2::rect::Rect;
use sdl2::keyboard::Keycode;
use sdl2::event::{Event,WindowEventId};

// use time;

use std::env;
use std::fmt;

mod cart;
mod mem_map;
mod cpu;
mod apu;
mod ppu;
mod opcodes;

use mem_map::*;
// use cpu::RunCondition;

pub struct Bus {
    ram: Box<[u8]>,
    cart: cart::Cart,
    apu: apu::APU,
    ppu: ppu::PPU,
}

fn main() {
    let rompath = env::args().nth(1).unwrap_or(String::from("smb.nes"));

    let sdl = sdl2::init().unwrap();
    let video = sdl.video().unwrap();
    let window = video.window("OxideNES", 256, 240)
        .position_centered()
        .opengl()
        .build()
        .unwrap();

    let mut renderer = window.renderer().build().unwrap();
    let mut texture = renderer.create_texture_streaming(PixelFormatEnum::RGB24,
                                                        256,
                                                        240).unwrap();
    let mut events = sdl.event_pump().unwrap();


    let cart = cart::Cart::new(&rompath);
    println!("{:#?}", cart);
    let chr_rom = cart::ChrRom::new(&rompath);
    let apu = apu::APU::new();

    let ppu = ppu::PPU::new(chr_rom);
    let cpubus = Bus {
        ram: vec![0; RAM_LEN as usize].into_boxed_slice(),
        cart: cart,
        apu: apu,
        ppu: ppu,
    };

    let pc = cpubus.cart.read_cart_u16(RESET_VECTOR_LOC);
    let mut cpu = cpu::CPU::new(cpubus, pc as u16);
    println!("{:#?}", cpu);

    // let mut ticks = 0;
    // TODO: re-add specific run conditions for debugging
    let mut nmi = false;
    'main: loop {

        let pc = cpu.program_counter;
        let instr = cpu.cpu_read_u8(pc);

/*
        // TODO: Move this to a specific debug output
if cpu.bus.ppu.framecount == 21 || cpu.bus.ppu.framecount == 22 {
        let tmp: u8 = cpu.status_reg.into();
        print!("{:#X}  I:{:02X}                  A:{:02X} X:{:02X} Y:{:02X}  P:{:02X}  \
                  SP:{:02X} CYC:{:>3} SL:{:} \r\n",
                 cpu.program_counter,
                 instr,
                 cpu.accumulator,
                 cpu.index_x,
                 cpu.index_y,
                 tmp,
                 cpu.stack_pointer,
                 cpu.cycle % 341,
                 cpu.bus.ppu.scanline,
                 ); //, self.status_reg);
}
*/

        cpu.cycle = cpu.cycle + (cpu::TIMING[instr as usize] * cpu::PPU_MULTIPLIER);

        if cpu.cycle >= 341 {
            cpu.cycle %= 341;
            nmi = cpu.bus.ppu.render_scanline();
        }
        cpu.execute_op(instr);

        // If the cycle count isn't > 1 yet
        // then the vblank flag wouldn't have been set at this point
        // since vblank is set on dot 1 of line 341
        if nmi && cpu.cycle > 2 {
            cpu.nmi();
            nmi = false;
        }


        if cpu.bus.ppu.scanline == 240 {
            // println!("screen 10,10 properly: {:#X}", cpu.bus.ppu.screen[10][10]);
            render_frame(&cpu.bus.ppu.screen, &mut renderer, &mut texture);
            for event in events.poll_iter() {
                match event {
                    Event::Quit {..} | Event::KeyDown { keycode: Some(Keycode::Escape), .. } => {
                        break 'main
                    }
                    _ => ()
                }
            }
        }

        if cpu.bus.ppu.scanline == -1 {


            let keys: Vec<Keycode> = events.keyboard_state().pressed_scancodes().
                            filter_map(Keycode::from_scancode).collect();

            for key in keys {
                match key {
                    Keycode::LCtrl => {
                        // panic!("Works..");
                        cpu.joy1 |= 1 << 0;
                    }
                    Keycode::LAlt => {
                        cpu.joy1 |= 1 << 1;
                    }
                    Keycode::Space => {
                        cpu.joy1 |= 1 << 2;
                    }
                    Keycode::Return => {
                        cpu.joy1 |= 1 << 3;
                    }
                    Keycode::Up => {
                        cpu.joy1 |= 1 << 4;
                    }
                    Keycode::Down => {
                        cpu.joy1 |= 1 << 5;
                    }
                    Keycode::Left => {
                        cpu.joy1 |= 1 << 6;
                    }
                    Keycode::Right => {
                        cpu.joy1 |= 1 << 7;
                    }
                    _ => ()// panic!("Unkown key {:?}", key),

                }

            }
        }

    }

}

fn render_frame(screen: &[[u32; 256]; 240],
                renderer: &mut sdl2::render::Renderer,
                texture: &mut sdl2::render::Texture,
                // events: &mut sdl2::EventPump,
                )
{
    //println!("Screen 10,10 {:#X}", screen[10][10]);
    texture.with_lock(None, |buffer: &mut [u8], pitch: usize| {
        // println!("pitch is: {:}", pitch);
        for row in 0..240 {
            let offset1 = row * pitch;
            for col in 0..256 {
                let offset2 = col * 3;
                let pixel = screen[row][col];
                let r = (pixel >> 16) as u8;
                let g = ((pixel >> 8) & 0xff) as u8;
                let b = (pixel & 0xff) as u8;

                buffer[offset1 + 0 + offset2] = r;
                buffer[offset1 + 1 + offset2] = g;
                buffer[offset1 + 2 + offset2] = b;

            }
        }
    }).unwrap();

    renderer.clear();
    renderer.copy(&texture, None, None);
    renderer.present();

}








impl fmt::Debug for Bus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "")
    }
}
