fn main() {
  let rt = tokio::runtime::Builder::new_multi_thread()
    .enable_all()
    .build()
    .unwrap();
  for (i, r) in unsafe { REG }.iter_mut().enumerate() {
    *r = i as u16;
  }
  loop {
    rt.block_on(async {
      let path = "/dev/ttyUSB1";
      let builder = tokio_serial::new(path, 9600);
      let server_serial = match tokio_serial::SerialStream::open(&builder) {
        Ok(serial) => serial,
        Err(err) => return println!("error opening {path} {err:?}"),
      };
      let server = server::rtu::Server::new(server_serial);
      println!("Simulating client. at {path}");
      server.serve_forever(|| Ok(MbServer)).await;
    });
    std::thread::sleep(std::time::Duration::from_millis(1000));
  }
}

use tokio_modbus::prelude::{Request, Response};
use tokio_modbus::server::{self, Service};

struct MbServer;

const REG_SIZE: usize = 10240;

static mut REG: [u16; REG_SIZE] = [0; REG_SIZE];

fn read_regs(a: u16, c: u16) -> Vec<u16> {
  unsafe { REG[a as usize..(a + c) as usize].into() }
}
fn write_regs(a: u16, vs: Vec<u16>) -> u16 {
  vs.iter().map(|v| write_reg(a, *v)).len() as u16
}
fn write_reg(a: u16, v: u16) -> u16 {
  unsafe { REG[a as usize] = v };
  v
}
fn read_coils(a: u16, c: u16) -> Vec<bool> {
  unsafe {
    REG[a as usize..(a + c) as usize]
      .iter()
      .map(|v| *v % 2 == 0)
      .collect()
  }
}
fn write_coil(a: u16, v: bool) -> bool {
  unsafe { REG[a as usize] = v as u16 }
  v
}
fn write_coils(a: u16, vs: Vec<bool>) -> u16 {
  vs.iter().map(|v| write_coil(a, *v)).len() as u16
}
use Response::*;
impl Service for MbServer {
  type Request = Request;
  type Response = Response;
  type Error = std::io::Error;
  type Future = core::future::Ready<Result<Self::Response, Self::Error>>;
  fn call(&self, req: Self::Request) -> Self::Future {
    print!("{req:?}");
    let res = match req {
      Request::ReadCoils(a, c) => ReadCoils(read_coils(a, c)),
      Request::ReadDiscreteInputs(a, c) => ReadDiscreteInputs(read_coils(a, c)),
      Request::WriteSingleCoil(a, v) => WriteSingleCoil(a, write_coil(a, v)),
      Request::WriteMultipleCoils(a, vs) => WriteMultipleCoils(a, write_coils(a, vs)),
      Request::ReadInputRegisters(a, c) => ReadInputRegisters(read_regs(a, c)),
      Request::ReadHoldingRegisters(a, c) => ReadHoldingRegisters(read_regs(a, c)),
      Request::WriteSingleRegister(a, v) => WriteSingleRegister(a, write_reg(a, v)),
      Request::WriteMultipleRegisters(a, vs) => WriteMultipleRegisters(a, write_regs(a, vs)),
      _re => Response::ReadInputRegisters([].into()),
    };
    println!(" => {res:?}");
    core::future::ready(Ok(res))
  }
}
