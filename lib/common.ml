(* Setup our monad *)
let (let*) = Lwt.bind
let return = Lwt.return

let parse_ipaddr = Unix.inet_addr_of_string
module IO = Lwt_io
module Unix = Lwt_unix

let localhost = Unix.ADDR_INET (parse_ipaddr "0.0.0.0", 1337)

exception SocketClosed

let server_wrapper server addr socket = 
    let* () = Lwt_io.printf "Client connected...\n" in
    try
        server addr socket
    with
        SocketClosed -> 
            Lwt_io.printf "Socket Closed\n"


let main name server = 
    let* _ = Lwt_io.printf "Starting\n" in
    let* _ = IO.establish_server_with_client_socket localhost server in
    let* _ = Lwt_io.printf "Started %s servfer...\n" name in
    let rec loop () = 
        let* () = Unix.sleep 1.0 in
        loop () 
    in
    loop ()

