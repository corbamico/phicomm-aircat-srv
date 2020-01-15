using System.Net;
using System.Net.Sockets;
using System.Net.Http;
using System.Threading.Tasks;

namespace aircat_srv_cs
{
    class RestSrv
    {
        Config _config;
        public RestSrv(Config conf) => _config = conf;
        public async Task RunAsync()
        {
            HttpListener listener = new HttpListener();
            listener.Start();
        }
    }
}