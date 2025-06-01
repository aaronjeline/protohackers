open Protohackers.Common

type message =
    | Insert of { timestamp : int; value : int }
    | Query of { t1 : int; t2 : int }
    [@@deriving show]

type parse_error =
    | InvalidDiscriminator of char
    [@@deriving show]

exception ParseError of parse_error


let parse_insert bs = 
    let timestamp = Bytes.get_int32_be bs 1 |> Int32.to_int in
    let  value = Bytes.get_int32_be bs 5 |> Int32.to_int in
    Insert { timestamp; value }

let parse_query bs = 
    let t1 = Bytes.get_int32_be bs 1 |> Int32.to_int in
    let t2 = Bytes.get_int32_be bs 5 |> Int32.to_int in
    Query { t1; t2}

let parse_message bs = 
    assert (Bytes.length bs = 9);
    let discriminator = Bytes.get bs 0 in
    match discriminator with
    | 'I' -> parse_insert bs
    | 'Q' -> parse_query bs
    | other -> raise (ParseError (InvalidDiscriminator other))


type response = int

let serialize_response (r:response) =
    let buf = Bytes.create 4 in
    let i32 = Int32.of_int r in
    Bytes.set_int32_be buf 0 i32;
    buf


exception SocketClosed

let read_message socket =
    let buf = Bytes.create 9 in
    let rec loop read =
        let* got = Unix.recv socket buf read (9 - read) [] in
        if got = 0 then
            raise SocketClosed;
        let read' = read + got in
        if read' = 9 then
            return ()
        else
            loop read'
    in
    let* () = loop 0 in
    return (parse_message buf)

module M = Map.Make(Int)

let execute_query map t1 t2 =
    let f timestamp value (sum, len) = 
        if timestamp >= t1 && timestamp <= t2 then
            (sum + value, len + 1)
        else
            (sum, len)
    in
    let (sum, len) = M.fold f map (0,0) in 
    sum / len

let eval_msg msg map = 
    match msg with
    | Insert { timestamp; value} ->
            (M.add timestamp value map, None)
    | Query { t1; t2 } ->
            (map, Some (execute_query map t1 t2))

let write_response rsp socket =
    let buf = serialize_response rsp in
    let rec loop ptr = 
        let to_send = (Bytes.length buf) - ptr in
        if to_send = 0 then
            return ()
        else
            let* sent = Unix.send socket buf ptr to_send [] in
            loop (ptr + sent)
    in loop 0

let rec server map addr socket = 
    let* msg = read_message socket in 
    let (map', resp) = eval_msg msg map in
    let* () = match resp with
    | Some resp -> write_response resp socket
    | None -> return () in
    server map' addr socket


let () = Lwt_main.run (main "stocks" (server M.empty))
