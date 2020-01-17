using System.Threading.Tasks;

namespace aircat_srv_cs
{
    class Program
    {
        static void Main(string[] args)
        {
            Config conf = Config.loadConfig("config.json");
            AircatSrv aircatsrv = new AircatSrv(conf);
            RestSrv restsrv = new RestSrv(conf);
            Task.WaitAny(aircatsrv.RunAsync(), restsrv.RunAsync());
        }
    }
}
