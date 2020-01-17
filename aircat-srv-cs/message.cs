using System.Buffers.Binary;
using System.Data.Common;
using System;
using System.Threading.Tasks;
using System.Net.Http;

namespace aircat_srv_cs
{
    class AirCatPacket
    {
        const int MIN_PACKET_LENGTH = 33;
        const int MAX_PACKET_LENGTH = 156;
        byte[] device_fixed;
        byte msg_type;
        byte[] mac;
        byte[] json;

        public byte[] Json { get => json; set => json = value; }

        /*Rawheader show as
           00 01 02 03 04 05 06 07   08  09 10 11  12 13 14 15
        00 -------unknown---------   0B  00 00 00  00 00 00 00
        16 ---------MAC-----------   len 00 00 typ
        */
        public static AirCatPacket From(byte[] bytes, int length)
        {
            if (length > bytes.Length || MIN_PACKET_LENGTH > length || length > MAX_PACKET_LENGTH)
            {
                return null;
            }
            byte len = bytes[24];
            byte begin = 28;
            byte end = (byte)(begin + len - 3);

            if ((begin > end) || (end > length))
            {
                return null;
            }
            AirCatPacket air = new AirCatPacket();
            air.device_fixed = new byte[16];
            air.mac = new byte[8];
            air.Json = new byte[len - 3];
            air.msg_type = bytes[27];

            Array.Copy(bytes, 0, air.device_fixed, 0, 16);
            Array.Copy(bytes, 16, air.mac, 0, 8);
            Array.Copy(bytes, begin, air.Json, 0, len - 3);
            return air;
        }
        public string ToInfluxLine()
        {
            try
            {
                string mac = String.Concat(Array.ConvertAll(this.mac, x => x.ToString("x2")));
                string json = System.Text.Encoding.Default.GetString(this.Json);
                AirMeasure air = System.Text.Json.JsonSerializer.Deserialize<AirMeasure>(json);
                return String.Format("aircat,mac=\"%s\" humidity=%s,temperature=%s,value=%s,hcho=%s", mac, air.humidity, air.temperature, air.value, air.hcho);
            }
            catch (Exception)
            {
                return null;
            }
            //return as "aircat,mac=\"%s\" humidity=%s,temperature=%s,value=%s,hcho=%s"
        }
    }
    class InfluxDb
    {
        private static readonly HttpClient _client;
        private static readonly string _uriTemplate;
        static InfluxDb()
        {
            _client = new HttpClient();
            _uriTemplate = "http://{0}/write?db=aircat";
        }
        public static async Task SendCmdLine(string addr, string air)
        {
            await Task.Yield();
            using (HttpContent content = new StringContent(air))
            {
                try
                {
                    await _client.PostAsync(String.Format(_uriTemplate, addr), content);
                }
                catch { }
            }
            return;
        }
    }

    struct AirMeasure
    {
        public string humidity { set; get; }

        public string temperature { set; get; }

        public string value { set; get; }

        public string hcho { set; get; }
    }
}