using System;
using System.Net;
using System.Threading.Tasks;

namespace aircat_srv_cs
{
    class RestSrv
    {
        Config _config;

        public RestSrv(Config conf) => _config = conf;
        public async Task RunAsync()
        {
            HttpListener listener = new HttpListener
            {
                Prefixes = { $"http://{_config.RESTServerAddr}/" }
            };
            listener.Start();
            Console.WriteLine("aircat restsrv run at {0}", _config.RESTServerAddr);
            while (true)
            {
                var ctx = await listener.GetContextAsync();
                var task = this.ProcesseRequest(ctx);
                if (task.IsFaulted)
                {
                    task.Wait();
                }
            }
        }
        private async Task ProcesseRequest(HttpListenerContext ctx)
        {
            await Task.Yield();
            var req = ctx.Request;
            using (var res = ctx.Response)
            {
                switch ((req.HttpMethod, req.Url.AbsolutePath))
                {
                    case ("GET", "/v1/aircat"):
                        var bytes = AircatSrv.LastPacket?.Json;
                        await res.OutputStream.WriteAsync(bytes, 0, bytes.Length);
                        res.StatusCode = (int)HttpStatusCode.OK;
                        break;
                    case ("PUT", "/v1/aircat"):
                        res.StatusCode = (int)HttpStatusCode.NoContent;
                        break;
                    default:
                        res.StatusCode = (int)HttpStatusCode.NotFound;
                        break;
                }
            }
        }
    }
}