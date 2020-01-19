using System.Net;
using System.Net.Sockets;
using System.Threading.Tasks;

namespace aircat_srv_cs
{
    class AircatSrv
    {
        Config _config;
        static AirCatPacket _lastPacket;
        static AircatDevice _lastDevice;

        internal static AirCatPacket LastPacket { get => _lastPacket; set => _lastPacket = value; }
        internal static AircatDevice LastDevice { get => _lastDevice; set => _lastDevice = value; }

        public AircatSrv(Config conf) => _config = conf;
        public async Task RunAsync()
        {
            if (this._config is null)
            {
                throw new System.ArgumentNullException(nameof(this._config));
            }
            IPEndPoint p = IPEndPoint.Parse(this._config.ServerAddr);
            TcpListener listener = new TcpListener(p);
            System.Console.WriteLine("aircat run at {0}", p.ToString());
            listener.Start();
            while (true)
            {
                TcpClient client = await listener.AcceptTcpClientAsync();
                AircatDevice conn = new AircatDevice(client, _config.InfluxdbServer);
                AircatSrv.LastDevice = conn;
                var task = conn.RunAsync();
                if (task.IsFaulted)
                {
                    task.Wait();
                }
            }
            //:-) non-stop
            //listener.Stop();
        }
        public static async Task SendContrlToDevice(byte[] json)
        {
            await _lastDevice?.SendBytesAsync(_lastPacket?.ToBytes(json));
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
        public async Task SendBytesAsync(byte[] bytes)
        {
            await _client.GetStream()?.WriteAsync(bytes, 0, bytes.Length);
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
                    try
                    {
                        var packet = AirCatPacket.From(buffer, nRead);
                        AircatSrv.LastPacket = packet;
                        var line = packet.ToInfluxLine();
                        var task = InfluxDb.SendCmdLine(_influxAddr, line);
                        if (task.IsFaulted)
                        {
                            task.Wait();
                        }
                    }
                    catch (System.Exception)
                    {
                        //System.Console.WriteLine("aircat client report exception: {0}", ex);
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