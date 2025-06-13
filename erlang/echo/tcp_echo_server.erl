-module(tcp_echo_server).
-behaviour(gen_server).

-export([start_link/1, start_link/2, stop/0]).

-export([init/1, handle_call/3, handle_cast/2, handle_info/2,
         terminate/2, code_change/3]).

-define(DEFAULT_PORT, 1337).

-record(state, {
          listen_socket,
          port,
          acceptor_pid,
          client_counter = 0
}).


% APi

start_link(Port) ->
    start_link(?MODULE, Port).

start_link(ServerName, Port) ->
    gen_server:start_link({local, ServerName}, ?MODULE,
                          [Port], []).

stop() ->
    gen_server:call(?MODULE, stop).

% callbacks
init([Port]) ->
    process_flag(trap_exit, true),
    {ok, _SupPid} = connection_sup:start_link(),
    case gen_tcp:listen(Port, [binary, {packet,line}, {active, false},
                               {reuseaddr, true}]) of
        {ok, ListenSocket} ->
            io:format("TCP Server started on port ~p~n", [Port]),
            Me = self(),
            AcceptorPid = spawn_link(fun() -> accept_loop(ListenSocket, Me) end),
            {ok, #state{listen_socket = ListenSocket,
                        port = Port,
                        acceptor_pid = AcceptorPid}};
        {error, Reason} ->
            {stop, Reason}
    end.

handle_call(stop, _From, State) ->
    {stop, normal, ok, State};
handle_call(_Request, _From, State) ->
    {reply, {error, unknown_call}, State}.

handle_cast({new_connection, Socket}, State) ->
    ClientId = State#state.client_counter + 1,
    case connection_sup:start_worker(Socket, ClientId) of
        {ok, WorkerPid} ->
            Status = gen_tcp:controlling_process(Socket, WorkerPid),
            io:format("Control switch: ~p~n", [Status]),
            connection_worker:mark_active(WorkerPid),
            io:format("Started worker for client ~p~n", [ClientId]),
            {noreply, State#state{client_counter = ClientId}};
        {error, Reason} ->
            io:format("Failed to start worker: ~p~n", [Reason]),
            gen_tcp:close(Socket),
            {noreply, State}
    end;
handle_cast(_Msg, State) ->
    {noreply, State}.

handle_info({'EXIT', Pid, Reason}, #state{acceptor_pid = Pid} = State) ->
    io:format("Acceptor process died: ~p~n", [Reason]),
    %% Restart acceptor
    Me = self(),
    NewAcceptorPid = spawn_link(fun() -> accept_loop(State#state.listen_socket, Me) end),
    {noreply, State#state{acceptor_pid = NewAcceptorPid}};
handle_info(Info, State) ->
    io:format("Unexpected message: ~p~n", [Info]),
    {noreply, State}.

terminate(_Reason, #state{listen_socket = ListenSocket}) ->
    gen_tcp:close(ListenSocket),
    ok.

code_change(_Old,State,_Extra) ->
    io:format("Code change!~n"),
    {ok, State}.

%% Internal Functions


accept_loop(ListenSocket, ServerPid) ->
    case gen_tcp:accept(ListenSocket) of
        {ok, Socket} ->
            gen_tcp:controlling_process(Socket, ServerPid),
            gen_server:cast(tcp_echo_server, {new_connection, Socket}),
            accept_loop(ListenSocket, ServerPid);
        {error, closed} ->
            ok;
        {error, Reason} ->
            io:format("Accept error: ~p~n", [Reason]),
            timer:sleep(1000),
            accept_loop(ListenSocket, ServerPid)
    end.

