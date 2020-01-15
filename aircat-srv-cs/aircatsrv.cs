using System.Net;
using System.Net.Sockets;
using System.Threading.Tasks;

namespace aircat_srv_cs
{
    class AircatSrv
    {
        Config _config;
        public AircatSrv(Config conf) => _config = conf;
        public async Task RunAsync()
        {
            if (!(this._config is null))
            {
                IPEndPoint p = IPEndPoint.Parse(this._config.ServerAddr);
                TcpListener listener = new TcpListener(p);
                System.Console.WriteLine("aircat run at {0}", p.ToString());
                listener.Start();
                while (true)
                {
                    TcpClient client = await listener.AcceptTcpClientAsync();
                    AircatDevice conn = new AircatDevice(client, _config.InfluxdbServer);
                    var task = conn.RunAsync();
                    if (task.IsFaulted)
                    {
                        task.Wait();
                    }
                }
            }
            throw new System.ArgumentNullException(nameof(this._config));
            //:-) non-stop
            //listener.Stop();
        }
    }
    class AircatDevice
    {
        TcpClient _client;
        string _influxAddr;

        string _addr;
        public AircatDevice(TcpClient client, string influxAddr)
        {
            _client = client;
            _influxAddr = influxAddr;
            _addr = _client.Client.RemoteEndPoint.ToString();
        }

        public async Task RunAsync()
        {
            System.Console.WriteLine("aircat client connect at {0}", _addr);
            var task = HandleAsync();
            try
            {
                await task;
            }
            catch (System.Exception)
            {
                //System.Console.WriteLine("aircat exception at {0}", ex.ToString());
            }
            finally
            {
                _client.Dispose(); _client = null;
                System.Console.WriteLine("aircat client disconnect at {0}", _addr);
            }
        }
        private async Task HandleAsync()
        {
            await Task.Yield();
            //we return immediately, and run in another thread, maybe.

            byte[] buffer = new byte[256];
            int nRead;

            using (var stream = _client.GetStream())
                while ((nRead = await stream.ReadAsync(buffer, 0, 256).ConfigureAwait(false)) != 0)
                {
                    var json = AirCatPacket.From(buffer, nRead)?.ToInfluxLine();
                    if (json != null)
                    {
                        await InfluxDb.SendJson(_influxAddr, json);
                    }
                }
        }
    }

    public class Config
    {
        public string ServerAddr { get; set; }
        public string RESTServerAddr { get; set; }

        public string InfluxdbServer { get; set; }
        public static Config loadConfig(string file)
        {
            string json = System.IO.File.ReadAllText(file);
            return System.Text.Json.JsonSerializer.Deserialize<Config>(json);
        }
    }
}