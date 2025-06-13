-module(connection_sup).
-behaviour(supervisor).

-export([start_link/0, start_worker/2, stop_worker/1, list_workers/0]).

-export([init/1]).

-define(SERVER, ?MODULE).

start_link() ->
    supervisor:start_link({local, ?SERVER}, ?MODULE, []).

% Start a new worker dynamically
start_worker(Socket, ClientId) ->
    supervisor:start_child(?SERVER, [Socket, ClientId]).

% Stop a worker
stop_worker(Pid) ->
    supervisor:terminate_child(?SERVER, Pid).

% list workers
list_workers() ->
    supervisor:which_children(?SERVER).

init([]) ->
    SupFlags = #{strategy => simple_one_for_one,
                 intensity => 5,
                 period => 5},

    ChildSpec = #{id => connection_worker,
                  start => {connection_worker, start_link, []},
                  restart => temporary,
                  shutdown => 5000,
                  type => worker,
                  modules => [connection_worker]},
    {ok, {SupFlags, [ChildSpec]}}.
