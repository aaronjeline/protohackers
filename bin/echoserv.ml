
(* Setup our monad *)
let (let*) = Lwt.bind
let return = Lwt.return

let parse_ipaddr = Unix.inet_addr_of_string
module IO = Lwt_io
module Unix = Lwt_unix

let localhost = Unix.ADDR_INET (parse_ipaddr "0.0.0.0", 1337)

let server _client_address socket = 
    let total = ref (Bytes.create 0) in
    let rec recv_loop () =
        let buf = Bytes.create 512 in
        let* got = Unix.recv socket buf 0 512 [] in
        if got = 0 then
            return ()
        else begin
            total := Bytes.cat !total (Bytes.sub buf 0 got);
            recv_loop () 
        end
    in
    let* () = recv_loop () in
    let rec send_loop start = 
        if start = Bytes.length !total then
            return ()
        else begin
            let* sent = Unix.send socket !total start (Bytes.length !total - start) [] in
            send_loop (start + sent)
        end in
    send_loop 0


    

let main () = 
    let* _ = Lwt_io.printf "Starting\n" in
    let* _ = IO.establish_server_with_client_socket localhost server in
    let* _ = Lwt_io.printf "Started echo servfer...\n" in
    let rec loop () = 
        let* () = Unix.sleep 1.0 in
        loop () 
    in
    loop ()

let () = Lwt_main.run (main ())
