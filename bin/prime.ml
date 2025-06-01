open Protohackers.Common

exception SocketClosed

let non_empty str = String.length str != 0

let ends_with c str = 
    let last_char = String.get str (String.length str - 1) in
    last_char = c

let rec unsnoc lst = 
    match lst with
    | [] -> failwith "Unsnock of empty list"
    | x::[] -> ([], x)
    | x::xs -> 
        let (xs', last) = unsnoc xs in 
        (x::xs', last)

let parse_input str =
    let terminal = ends_with '\n' str in
    let lines = 
            String.split_on_char '\n' str
            |> List.filter non_empty in
    if terminal then
        (lines, "")
    else
        unsnoc lines 
     

let read_input last socket : (string list * string) Lwt.t = 
    let buf = Bytes.create 512 in
    let* got = Unix.recv socket buf 0 512 [] in
    if got = 0 then
            raise SocketClosed;
    let as_str = Bytes.sub buf 0 got |> Bytes.to_string in 
    return (parse_input (last ^ as_str))

let send_response response socket = 
    let* () = Lwt_io.printf "Sending `%s`\n" response in
    let buf = Bytes.of_string (String.cat response "\n") in
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


type parse_error =
        | ExpectedString
        | ExpectedInt
        | ExpectedExact of string
        | ExpectedObject
        | JsonParseError
        [@@deriving show]

exception ParseError of (parse_error * string)


let show_pe (pe, str) =
        let x = show_parse_error pe in
        Printf.sprintf "%s:%s" x str

exception PE of parse_error

let of_key dict name f =
    f (List.assoc name dict)

let expect_string json = 
    match json with
    | `String x -> x
    | _ -> raise (PE ExpectedString)

let expect_int json = 
    match json with
    | `Int i -> `Int i
    | `Float f -> `Float f
    | `Intlit x -> `Bigint x
    | _ -> raise (PE ExpectedInt)

let exact_string x json = 
    let p = expect_string json in 
    if p = x then
        ()
    else
        raise (PE (ExpectedExact x))

let expect_object json =
    match json with
    | `Assoc members -> members
    | _ -> raise (PE ExpectedObject)

let parse_input line =
    try
        let json = Yojson.Safe.from_string line in
        let dict = expect_object json in
        of_key dict "method" (exact_string "isPrime");
        of_key dict "number" expect_int
    with
        | PE err -> raise (ParseError (err, line))
        | Yojson.Json_error _ -> raise (ParseError (JsonParseError, line))


let is_prime x = 
    let x = match x with
    | `Float _ -> Z.of_int 1
    | `Bigint x -> Z.of_string x
    | `Int x  -> Z.of_int x
    in
    let rec loop i = 
      let stop = Z.sqrt x in
      if i = stop then
        true
      else if (Z.congruent x Z.zero i) then
        false
      else
        loop (Z.add i Z.one)
    in
    if x <= Z.one || x = Z.of_int 4 then
      false
    else if x = Z.of_int 2 || x = Z.of_int 3 then
      true
    else
      loop (Z.of_int 2)


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
            Lwt_io.printf "Connection closed\n"
    | ParseError pe ->
            let msg = show_pe pe in
            let* () = Lwt_io.printf "%s\n" msg in
            send_response "malformed!" socket
    | _ ->
            let* () = Lwt_io.printf "Unexpected Exception: %s\n" (Printexc.to_string exn) in
            send_response "malformed!" socket

let server _ socket  =
    let rec loop slop = 
        let* () = Lwt_io.printf "Client connected\n" in
        let* (lines, leftover) = read_input slop socket in
        let* () = Lwt_list.iter_s (process_input socket) lines in
        loop leftover
    in Lwt.catch
    (fun () ->  loop "" )
    (fun exn -> handle_failure exn socket)



let () = Lwt_main.run (main "prime" server)
