using System;

namespace aircat_srv_cs
{
    class Program
    {
        static async System.Threading.Tasks.Task Main(string[] args)
        {
            Config conf = Config.loadConfig("config.json");
            AircatSrv aircatsrv = new AircatSrv(conf);
            await aircatsrv.RunAsync().ConfigureAwait(false);
        }
    }
}
