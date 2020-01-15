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

        /*Rawheader show as
           00 01 02 03 04 05 06 07   08  09 10 11  12 13 14 15
        00 -------unknown---------   0B  00 00 00  00 00 00 00
        16 ---------MAC-----------   len 00 00 typ
        */
        public static AirCatPacket From(byte[] bytes, int length)
        {

            if (length > bytes.Length || MIN_PACKET_LENGTH < length || bytes.Length > length)
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
            air.json = new byte[len - 3];
            air.msg_type = bytes[27];

            Array.Copy(bytes, 0, air.device_fixed, 0, 16);
            Array.Copy(bytes, 16, air.mac, 0, 8);
            Array.Copy(bytes, begin, air.json, 0, len - 3);
            return air;
        }
        public string ToInfluxLine()
        {
            try
            {
                string mac = String.Concat(Array.ConvertAll(this.mac, x => x.ToString("x2")));
                string json = System.Text.Encoding.Default.GetString(this.json);
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
        static InfluxDb()
        {
            _client = new HttpClient();
        }
        public static async Task SendJson(string url, string air)
        {
            Task.Yield();

            using (HttpContent content = new StringContent(air))
            {
                await _client.PostAsync(url, content);
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