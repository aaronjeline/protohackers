-module(tcp_echo_sup).
-behaviour(supervisor).

-export([start_link/0, start_link/1, terminate/0]).

-export([init/1]).

-define(SERVER, ?MODULE).
-define(DEFAULT_PORT, 1337).

% API

start_link() ->
    start_link(?DEFAULT_PORT).

start_link(Port) ->
    supervisor:start_link({local, ?SERVER}, ?MODULE, [Port]).

terminate() ->
    supervisor:terminate_child(tcp_echo_server, ?SERVER),
    ok.


% Callbacks


init([Port]) ->
    SupFlags = #{strategy => one_for_one,
                 intensity => 5,
                 period => 10},
    ChildSpecs = [#{id => tcp_echo_server,
                    start => {tcp_echo_server, start_link, [tcp_echo_server, Port]},
                    restart => permanent,
                    shutdown => 5000,
                    type => worker,
                    modules => [tcp_echo_server]}],

    {ok, {SupFlags, ChildSpecs}}.
