extern crate env_logger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate prost_derive;
extern crate futures;
extern crate tokio_core;
extern crate http;
extern crate tower_h2;
extern crate tower_grpc;
#[macro_use]
extern crate mysql_async;


use std::env;
use std::process;

use futures::{future, Future, Stream};
use tokio_core::net::TcpListener;
use tokio_core::reactor::{Core, Handle};
use tower_grpc::{Request, Response};
use mysql_async::prelude::*;

//
// Include the code generated by the protobuf preprocessor, scope it by 'services'
//
pub mod services {
   include!(concat!(env!("OUT_DIR"), "/services.rs"));
}

//
// Make SimpleServer cloneable (and debuaggable?)
#[derive(Clone, Debug)]
struct SimpleServer {
  mysql_pool: mysql_async::Pool,
}

//
// Constructor for SimpleServer; the mysql_async pool needs the
// reactor handle to register futures.
//
impl SimpleServer {

  fn new (mysql_url: String, reactor_handle: &Handle) -> SimpleServer 
  { 
     SimpleServer {
       mysql_pool: mysql_async::Pool::new(mysql_url, reactor_handle),
     }
  }


  //
  // Implement DB call and return 'impl Future'
  //
  fn simple_db_call (&mut self, number: u32) -> impl Future<Item = Response<services::NumberMessage>, Error = tower_grpc::Error> {
    // get the mysql connection
    let connection = self.mysql_pool.get_conn();

    // take connection and executre query, map error into grpc
    let query = connection.and_then(|conn| conn.prep_exec("SELECT :number AS TEST", params!{ number} ));

    // get result set and reduce it
    let result = query.and_then(|result| result.reduce_and_drop(0u32, |mut _lastval, row| {
          let (res,) : (u32,) = mysql_async::from_row(row);
          res
        })).map_err(|_| make_grpc_error(tower_grpc::Status::INTERNAL));

    result.then(|result| match result {
        Ok(result) => futures::future::ok(Response::new(services::NumberMessage { number: 0 })),
        Err(e) => futures::future::err(make_grpc_error(tower_grpc::Status::INTERNAL)),
    })
  }
}

//
// Utility function to make simple errors for the service
//
fn make_grpc_error (status: tower_grpc::Status) -> tower_grpc::Error {
    tower_grpc::Error::Grpc(status, http::HeaderMap::new())
}


impl services::server::SimpleService for SimpleServer {

  type GetNumberFuture = future::FutureResult<Response<services::NumberMessage>, tower_grpc::Error>;

  fn get_number (&mut self, request:Request<services::NumberMessage>) -> Self::GetNumberFuture {

    // fetch the number from the incoming request
    let number = request.into_inner().number;

    // future::err(make_grpc_error(tower_grpc::Status::INTERNAL))
    let future = self.simple_db_call(number);
    future
  }

}

fn main() {
    println!("Hello, world!");
}
