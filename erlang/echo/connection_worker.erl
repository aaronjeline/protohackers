-module(connection_worker).
-behavior(gen_server).

-export([start_link/2, get_info/1, send_data/2,
         mark_active/1]).

-export([init/1, handle_call/3, handle_cast/2, handle_info/2,
         terminate/2, code_change/3]).

-record(state, {
          socket,
          client_id,
          start_time
         }).


% API

start_link(Socket, ClientId) ->
    gen_server:start_link(?MODULE, [Socket,ClientId], []).

get_info(Pid) ->
    gen_server:call(Pid, get_info).

send_data(Pid, Data) ->
    gen_server:cast(Pid, {send_Data, Data}).

mark_active(Pid) ->
    gen_server:cast(Pid, mark_active).

% gen_server callbacks

init([Socket,ClientId]) ->
    inet:setopts(Socket, [{active, once}]),
    io:format("Worker started for client ~p~n", [ClientId]),
    {ok, #state{socket = Socket,
                client_id = ClientId,
                start_time = erlang:timestamp()}}.

handle_call(get_info, _From, State) ->
    Info = #{client_id => State#state.client_id,
             start_time => State#state.start_time,
             socket => State#state.socket},
    {reply, Info, State};
handle_call(_Request,_From,State) ->
    {reply, {error, unknown_call}, State}.

handle_cast(mark_active, State) ->
    inet:setopts(State#state.socket, [{active, once}]),
    {noreply, State};
handle_cast({send_data,Data},State) ->
    gen_tcp:send(State#state.socket, Data),
    {noreply, State};
handle_cast(_Msg, State) ->
    {noreply, State}.

handle_info({tcp, Socket, Data}, State) ->
    io:format("client ~p send: ~p~n", [State#state.client_id, Data]),
    gen_tcp:send(Socket, [Data]),
    inet:setopts(Socket, [{active, once}]),
    {noreply, State};
handle_info({tcp_closed, _Socket, _Reason}, State) ->
    io:format("Client ~p disconnected~n", [State#state.client_id]),
    {stop, normal, State};
handle_info({tcp_error, _Socket, Reason}, State) ->
    io:fomrat("TCP Error for client ~p: ~p~n", [State#state.client_id, Reason]),
    {stop, {tcp_error, Reason}, State};
handle_info(_Info, State) ->
    {noreply, State}.

terminate(Reason, State) ->
    io:format("Worker for client ~p temrinating: ~p~n",
              [State#state.client_id, Reason]),
    gen_tcp:close(State#state.socket),
    ok.

code_change(_,State,_) ->
    {ok, State}.



