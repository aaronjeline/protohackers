open Protohackers.Common

exception SocketClosed

let non_empty str = String.length str != 0

let read_input socket = 
    let buf = Bytes.create 512 in
    let* () = Lwt_io.printf "Reading from socket...\n" in
    let* got = Unix.recv socket buf 0 512 [] in
    let* () = Lwt_io.printf "Read %d bytes\n" got in
    if got = 0 then
        raise SocketClosed;
    Bytes.sub buf 0 got
    |> Bytes.to_string 
    |> String.split_on_char '\n' 
    |> List.filter non_empty
    |> return


let send_response response socket = 
    let buf = Bytes.of_string response in
    let to_send = Bytes.length buf in
    let* () = Lwt_io.printf "Sending %d bytes\n" to_send in
    let rec loop start =
        if to_send - start <= 0 then
            return ()
        else 
            let* () = Lwt_io.printf "send...\n" in
            let* sent = Unix.send socket buf start to_send [] in
            let* () = Lwt_io.printf "sent %d bytes\n" sent in
            loop (to_send + sent)
    in
    loop 0



exception ExpectedString
exception ExpectedInt
exception ExpectedExact of string
exception ExpectedObject

let of_key dict name f =
    f (List.assoc name dict)

let expect_string json = 
    match json with
    | `String x -> x
    | _ -> raise ExpectedString

let expect_int json = 
    match json with
    | `Int i -> `Int i
    | `Float f -> `Float f
    | _ -> raise ExpectedInt

let exact_string x json = 
    let p = expect_string json in 
    if p = x then
        ()
    else
        raise (ExpectedExact x)

let expect_object json =
    match json with
    | `Assoc members -> members
    | _ -> raise ExpectedObject

let parse_input line =
    let json = Yojson.Safe.from_string line in
    let dict = expect_object json in
    of_key dict "method" (exact_string "isPrime");
    of_key dict "number" expect_int


let is_prime x = 
    match x with
    | `Int x -> not ((x mod 2) == 0)
    | _ -> false


let encode_response is_prime =
    let json = `Assoc [
        ("method", `String "isPrime");
        ("prime", `Bool is_prime)] in
    Yojson.Safe.to_string json


let process_input socket line =
    let parsed = parse_input line in
    let response = encode_response (is_prime parsed) in
    send_response response socket


let handle_failure exn socket =
    match exn with
    | SocketClosed ->
            Lwt_io.eprintf "Connection closed\n"
    | _ ->
            let* () = Lwt_io.eprintf "Exception: %s\n" (Printexc.to_string exn) in
            send_response "malformed!\n" socket


let server _ socket  =
    let rec loop () = 
        let* lines = read_input socket in
        let* () = Lwt_list.iter_s (process_input socket) lines in
        loop ()
    in Lwt.catch
    (fun () ->  loop () )
    (fun exn -> handle_failure exn socket)



let () = Lwt_main.run (main "prime" server)
