// Le passeur : quand un personnage rend visite à une autre session, il le fait
// voler, en vrai, d'une fenêtre de terminal à l'autre. passeur.ps1 le compile une
// fois en passeur.exe (dans %LOCALAPPDATA%\Diorama) et le lance ; il lit ce que le
// mod écrit sur le disque, rien de plus.
//
// Sobre au repos : un coup d'œil à la date du stockage tous les quarts de seconde ;
// quand elle a changé (deux fois par seconde avec une dizaine de sessions), une
// lecture et un balayage qui isole les visites sans décoder le reste. Une visite
// n'est décodée que si son texte a changé ; les scènes, qu'au moment d'un vol.
//
// Ce qu'il lit :
//   ~/.claude/plugins/store/diorama*.json  les visites (visit:<session>), leur phase,
//                                          le sprite et la scène du visiteur ;
//                                          la scène de chaque session (presence)
//   ~/.claude/sessions/<pid>.json           la session de chaque processus
// La fenêtre d'une session : s'attacher un instant à sa console, dont le
// propriétaire est la fenêtre de Windows Terminal (ou conhost lui-même).
//
// C# 5 : celui que compile Add-Type sous Windows PowerShell 5.1.

using System;
using System.Collections.Generic;
using System.Diagnostics;
using System.Drawing;
using System.IO;
using System.Runtime.InteropServices;
using System.Text;
using System.Web.Script.Serialization;
using System.Windows.Forms;

namespace Diorama
{
    static class Win
    {
        [DllImport("user32")] public static extern IntPtr SetProcessDpiAwarenessContext(IntPtr value);
        [DllImport("kernel32")] public static extern bool FreeConsole();
        [DllImport("kernel32")] public static extern bool AttachConsole(uint pid);
        [DllImport("kernel32")] public static extern IntPtr GetConsoleWindow();
        [DllImport("kernel32")] public static extern bool SetConsoleCtrlHandler(IntPtr handler, bool add);
        [DllImport("user32")] public static extern IntPtr GetAncestor(IntPtr hwnd, uint flags);
        [DllImport("user32", CharSet = CharSet.Unicode)] public static extern int GetClassName(IntPtr hwnd, StringBuilder name, int size);
        [DllImport("user32")] public static extern bool IsWindowVisible(IntPtr hwnd);
        [DllImport("user32")] public static extern bool IsIconic(IntPtr hwnd);
        [DllImport("user32")] public static extern bool GetClientRect(IntPtr hwnd, out Rect rect);
        [DllImport("user32")] public static extern bool ClientToScreen(IntPtr hwnd, ref Pt point);
        [DllImport("user32")] public static extern uint GetDpiForWindow(IntPtr hwnd);
        [DllImport("user32")] public static extern IntPtr WindowFromPoint(Pt point);
        [DllImport("user32")] public static extern IntPtr MonitorFromPoint(Pt point, uint flags);
        [DllImport("user32")] public static extern bool GetMonitorInfo(IntPtr monitor, ref MonitorInfo info);
        [StructLayout(LayoutKind.Sequential)] public struct MonitorInfo { public int Size; public Rect Monitor, Work; public uint Flags; }
        [DllImport("kernel32", CharSet = CharSet.Unicode)] public static extern IntPtr CreateFile(string name, uint access, uint share, IntPtr security, uint creation, uint flags, IntPtr template);
        [DllImport("kernel32")] public static extern bool GetConsoleScreenBufferInfo(IntPtr output, out BufferInfo info);
        [DllImport("kernel32")] public static extern bool CloseHandle(IntPtr handle);
        [DllImport("kernel32")] public static extern IntPtr OpenProcess(uint access, bool inherit, uint pid);
        [DllImport("kernel32")] public static extern bool GetExitCodeProcess(IntPtr process, out uint code);

        [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
        [StructLayout(LayoutKind.Sequential)] public struct Pt { public int X, Y; }
        [StructLayout(LayoutKind.Sequential)]
        public struct BufferInfo
        {
            public short SizeX, SizeY, CursorX, CursorY;
            public ushort Attributes;
            public short Left, Top, Right, Bottom, MaxX, MaxY;
        }
    }

    /// <summary>Le personnage en marche vers la droite, tel que le mod l'a découpé (hooks/flight.ts).</summary>
    public class Sprite
    {
        public int W, H, Top;
        public Dictionary<char, Color> Colours = new Dictionary<char, Color>();
        public List<string> Frames = new List<string>();

        public static Sprite From(object raw)
        {
            var d = raw as Dictionary<string, object>;
            if (d == null) return null;
            var s = new Sprite { W = Json.Int(d, "w"), H = Json.Int(d, "h"), Top = Json.Int(d, "top") };
            var colours = Json.Get(d, "colours") as Dictionary<string, object>;
            var frames = Json.Get(d, "frames") as object[];
            if (colours == null || frames == null || s.W <= 0 || s.H <= 0) return null;
            foreach (var kv in colours)
            {
                int c = Convert.ToInt32(kv.Value);
                s.Colours[kv.Key[0]] = Color.FromArgb(c >> 16 & 255, c >> 8 & 255, c & 255);
            }
            foreach (var f in frames)
            {
                var text = f as string;
                if (text != null && text.Length == s.W * s.H) s.Frames.Add(text);
            }
            return s.Frames.Count > 0 ? s : null;
        }
    }

    /// <summary>La scène d'une session : ses colonnes depuis la gauche, ses lignes (une ligne, deux pixels).</summary>
    public class Scene
    {
        public int Cols, Rows;

        public static Scene From(object raw)
        {
            var d = raw as Dictionary<string, object>;
            if (d == null) return null;
            var s = new Scene { Cols = Json.Int(d, "cols"), Rows = Json.Int(d, "rows") };
            return s.Cols > 0 && s.Rows > 0 ? s : null;
        }
    }

    static class Json
    {
        public static object Get(Dictionary<string, object> d, string key)
        {
            object v;
            return d != null && d.TryGetValue(key, out v) ? v : null;
        }
        public static string Str(Dictionary<string, object> d, string key) { return Get(d, key) as string; }
        public static int Int(Dictionary<string, object> d, string key)
        {
            var v = Get(d, key);
            try { return v == null ? 0 : Convert.ToInt32(v); } catch { return 0; }
        }
    }

    /// <summary>Où se tient une scène à l'écran : son bord droit, son haut, la taille d'une cellule (pixels de l'écran).</summary>
    public class Place
    {
        public IntPtr Window;
        public double Right, Top, Bottom, CellW, CellH;

        /// <summary>Si c'est bien sa fenêtre qu'on voit au coin bas-droit de sa scène, et pas une autre posée dessus.</summary>
        public bool InSight()
        {
            var at = new Win.Pt { X = (int)(Right - 2 * CellW), Y = (int)(Bottom - CellH) };
            return Win.GetAncestor(Win.WindowFromPoint(at), 2) == Window; // GA_ROOT
        }
    }

    static class Places
    {
        // Windows Terminal, à 96 ppp : la barre d'onglets, la marge autour du texte,
        // la barre de défilement (mesurés le 2026-10-13, réglages par défaut)
        const double TABS = 31, PAD = 8, SCROLLBAR = 16;
        // les lignes de Claude Code sous le bandeau : une vide, le cadre du prompt et sa ligne, le pied
        const int BELOW = 5;

        /// <summary>La fenêtre de la session `pid` et sa grille ; null si elle n'en a pas (une session de fond), ou réduite.</summary>
        public static Place Of(uint pid, Scene scene)
        {
            if (scene == null) return null;
            IntPtr window;
            int cols, rows;
            if (!Attach(pid, out window, out cols, out rows) || cols <= 0 || rows <= 0) return null;
            if (!Win.IsWindowVisible(window) || Win.IsIconic(window)) return null;
            var name = new StringBuilder(256);
            Win.GetClassName(window, name, 256);
            bool terminal = name.ToString() == "CASCADIA_HOSTING_WINDOW_CLASS";
            if (!terminal && name.ToString() != "ConsoleWindowClass") return null;
            Win.Rect client;
            Win.GetClientRect(window, out client);
            var origin = new Win.Pt();
            Win.ClientToScreen(window, ref origin);
            double k = Win.GetDpiForWindow(window) / 96.0;
            if (k <= 0) k = 1;
            // conhost : la zone cliente est la grille ; Windows Terminal l'entoure d'onglets et de marges
            double tabs = terminal ? TABS * k : 0, pad = terminal ? PAD * k : 0, bar = terminal ? SCROLLBAR * k : 0;
            double cellW = Math.Floor((client.Right - 2 * pad - bar) / cols);
            double cellH = Math.Floor((client.Bottom - tabs - 2 * pad) / rows);
            if (cellW < 2 || cellH < 4) return null;
            int sceneTop = Math.Max(0, rows - BELOW - scene.Rows);
            return new Place
            {
                Window = window, CellW = cellW, CellH = cellH,
                Right = origin.X + pad + scene.Cols * cellW,
                Top = origin.Y + tabs + pad + sceneTop * cellH,
                Bottom = origin.Y + tabs + pad + (sceneTop + scene.Rows) * cellH,
            };
        }

        /// <summary>S'attache un instant à la console de `pid` : sa fenêtre, sa taille en cellules.</summary>
        static bool Attach(uint pid, out IntPtr window, out int cols, out int rows)
        {
            window = IntPtr.Zero;
            cols = rows = 0;
            Win.FreeConsole();
            if (!Win.AttachConsole(pid)) return false;
            try
            {
                Win.SetConsoleCtrlHandler(IntPtr.Zero, true); // un Ctrl+C dans cette fenêtre-là n'est pas pour nous
                window = Win.GetAncestor(Win.GetConsoleWindow(), 3); // GA_ROOTOWNER
                IntPtr output = Win.CreateFile("CONOUT$", 0xC0000000, 3, IntPtr.Zero, 3, 0, IntPtr.Zero);
                if (output == new IntPtr(-1)) return false;
                Win.BufferInfo info;
                bool ok = Win.GetConsoleScreenBufferInfo(output, out info);
                Win.CloseHandle(output);
                if (!ok) return false;
                cols = info.Right - info.Left + 1;
                rows = info.Bottom - info.Top + 1;
                return window != IntPtr.Zero;
            }
            finally
            {
                Win.FreeConsole();
            }
        }
    }

    /// <summary>Un vol : une petite fenêtre sans bord, que les clics traversent, qui ne prend jamais la main.</summary>
    public class Flight : Form
    {
        static readonly Color Key = Color.FromArgb(255, 0, 255);
        const double Shrink = 0.25; // sa taille en vol, rapportée à celle qu'il a dans la scène
        readonly Sprite sprite;
        readonly Place from, to;
        readonly double x0, y0, x1, y1, height, seconds;
        readonly bool mirror;
        readonly Stopwatch clock = Stopwatch.StartNew();
        readonly Timer timer = new Timer { Interval = 15 };
        readonly Dictionary<char, SolidBrush> brushes = new Dictionary<char, SolidBrush>();
        readonly Dictionary<int, Bitmap> images = new Dictionary<int, Bitmap>();
        double pixelW, pixelH;
        int frame = -1;

        public Flight(Sprite sprite, Place from, Place to)
        {
            this.sprite = sprite;
            this.from = from;
            this.to = to;
            // ses pieds au bord droit de chaque scène, à la hauteur où il marche
            x0 = from.Right;
            y0 = from.Top + (sprite.Top + sprite.H) * from.CellH / 2;
            x1 = to.Right;
            y1 = to.Top + (sprite.Top + sprite.H) * to.CellH / 2;
            double distance = Math.Sqrt((x1 - x0) * (x1 - x0) + (y1 - y0) * (y1 - y0));
            // un saut, pas une catapulte : son sommet reste dans le tiers bas des deux écrans, et un petit bond au moins
            double tall = sprite.H * Math.Max(1, Math.Round(Math.Max(from.CellH, to.CellH) / 2 * Shrink));
            double ceiling = Math.Max(LowThird(x0, y0), LowThird(x1, y1)) + tall;
            double hop = 2 * tall;
            height = Math.Max(hop, Math.Min(0.3 * distance, Highest(y0, y1, ceiling)));
            seconds = Math.Min(2.2, Math.Max(1.0, 0.8 + distance / 3000));
            mirror = x1 < x0;
            foreach (var kv in sprite.Colours) brushes[kv.Key] = new SolidBrush(kv.Value);

            FormBorderStyle = FormBorderStyle.None;
            ShowInTaskbar = false;
            TopMost = true;
            StartPosition = FormStartPosition.Manual;
            BackColor = Key;
            TransparencyKey = Key;
            DoubleBuffered = true;
            Opacity = 0;
            timer.Tick += delegate { Step(); };
            Step();
            timer.Start();
        }

        /// <summary>Le haut du tiers bas de l'écran (sa zone de travail) qui contient ce point.</summary>
        static double LowThird(double x, double y)
        {
            var info = new Win.MonitorInfo { Size = Marshal.SizeOf(typeof(Win.MonitorInfo)) };
            if (!Win.GetMonitorInfo(Win.MonitorFromPoint(new Win.Pt { X = (int)x, Y = (int)y }, 2), ref info)) return y; // MONITOR_DEFAULTTONEAREST
            return info.Work.Top + (info.Work.Bottom - info.Work.Top) * 2.0 / 3;
        }

        /// <summary>La plus grande hauteur d'arc entre y0 et y1 dont aucun point ne monte au-dessus de `ceiling`.</summary>
        static double Highest(double y0, double y1, double ceiling)
        {
            double lo = 0, hi = 4000;
            for (int i = 0; i < 30; i++)
            {
                double h = (lo + hi) / 2, top = double.MaxValue;
                for (int k = 0; k <= 50; k++)
                {
                    double e = k / 50.0;
                    top = Math.Min(top, y0 + (y1 - y0) * e - h * 4 * e * (1 - e));
                }
                if (top >= ceiling) lo = h; else hi = h;
            }
            return lo;
        }

        protected override bool ShowWithoutActivation { get { return true; } }

        protected override CreateParams CreateParams
        {
            get
            {
                var p = base.CreateParams;
                // fenêtre d'outil (ni barre des tâches ni Alt+Tab), traversée par les clics, jamais active, au-dessus de tout
                p.ExStyle |= 0x80 | 0x20 | 0x08000000 | 0x80000 | 0x8;
                return p;
            }
        }

        void Step()
        {
            double t = clock.Elapsed.TotalSeconds / seconds;
            if (t >= 1)
            {
                timer.Stop();
                Close();
                return;
            }
            double e = 0.5 - 0.5 * Math.Cos(Math.PI * t); // part doucement, se pose doucement
            // au quart de sa taille dans la scène, en pixels entiers pour rester net
            pixelW = Math.Max(1, Math.Round((from.CellW + (to.CellW - from.CellW) * e) * Shrink));
            pixelH = Math.Max(1, Math.Round((from.CellH + (to.CellH - from.CellH) * e) / 2 * Shrink));
            double x = x0 + (x1 - x0) * e;
            double y = y0 + (y1 - y0) * e - height * 4 * e * (1 - e);
            int w = (int)Math.Ceiling(sprite.W * pixelW), h = (int)Math.Ceiling(sprite.H * pixelH);
            // déplacer une fenêtre ne la redessine pas : seulement quand son image ou sa taille change
            bool resized = w != Width || h != Height;
            SetBounds((int)Math.Round(x - w / 2.0), (int)Math.Round(y - h), w, h);
            double opacity = Math.Round(0.92 * Math.Min(1, Math.Min(t / 0.08, (1 - t) / 0.1)), 2);
            if (opacity != Opacity) Opacity = opacity;
            int next = (int)(clock.Elapsed.TotalSeconds / 0.1) % sprite.Frames.Count;
            if (resized) Forget();
            if (resized || next != frame)
            {
                frame = next;
                Invalidate();
            }
        }

        protected override void OnPaint(PaintEventArgs e)
        {
            Bitmap image;
            if (!images.TryGetValue(frame, out image))
            {
                images[frame] = image = Draw(sprite.Frames[frame], Width, Height); // une taille constante (deux fenêtres pareilles) : huit dessins par vol
            }
            e.Graphics.DrawImageUnscaled(image, 0, 0);
        }

        Bitmap Draw(string pixels, int w, int h)
        {
            var image = new Bitmap(Math.Max(1, w), Math.Max(1, h));
            using (var g = Graphics.FromImage(image))
            {
                g.Clear(Key);
                for (int y = 0; y < sprite.H; y++)
                {
                    int top = (int)Math.Round(y * pixelH), bottom = (int)Math.Round((y + 1) * pixelH);
                    for (int x = 0; x < sprite.W; x++)
                    {
                        SolidBrush brush;
                        if (!brushes.TryGetValue(pixels[y * sprite.W + (mirror ? sprite.W - 1 - x : x)], out brush)) continue;
                        int left = (int)Math.Round(x * pixelW), right = (int)Math.Round((x + 1) * pixelW);
                        g.FillRectangle(brush, left, top, right - left, bottom - top);
                    }
                }
            }
            return image;
        }

        void Forget()
        {
            foreach (var image in images.Values) image.Dispose();
            images.Clear();
        }

        protected override void Dispose(bool disposing)
        {
            if (disposing)
            {
                timer.Dispose();
                Forget();
                foreach (var b in brushes.Values) b.Dispose();
            }
            base.Dispose(disposing);
        }
    }

    /// <summary>Le passeur lui-même : regarde les visites, fait voler qui part et qui revient.</summary>
    public class Passeur : ApplicationContext
    {
        class Seen { public string Raw, Host, Phase; public Dictionary<string, object> Visit; }

        static string Home = Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), ".claude");
        static readonly string Log = Path.Combine(Path.GetTempPath(), "diorama-passeur.log");
        public const string Post = "Local\\DioramaPasseur"; // un seul passeur par machine (passeur.ps1 le regarde aussi)
        readonly JavaScriptSerializer json = new JavaScriptSerializer { MaxJsonLength = int.MaxValue };
        readonly Timer timer = new Timer { Interval = 250 };
        readonly string source;
        readonly DateTime sourceTime;
        Dictionary<string, Seen> seen = new Dictionary<string, Seen>();
        bool seeded;
        string storeFile;
        DateTime storeTime;
        int ticks, lonely;

        /// <summary>
        /// passeur.exe &lt;passeur.cs&gt; [racine] : le passeur à demeure, tant qu'une session Claude est
        /// ouverte et que son code (passeur.cs) ne change pas. La racine : le dossier .claude qu'il lit,
        /// un autre que le vrai pour un banc d'essai (qui prend alors un autre poste que le vrai).
        /// </summary>
        [STAThread]
        public static int Main(string[] args)
        {
            if (args.Length < 1) return 2;
            string root = args.Length > 1 ? args[1] : null;
            if (!string.IsNullOrEmpty(root)) Home = root;
            string name = Post + (string.IsNullOrEmpty(root) ? "" : "." + Math.Abs(root.ToLowerInvariant().GetHashCode()));
            bool mine;
            using (var post = new System.Threading.Mutex(true, name, out mine))
            {
                if (!mine)
                {
                    try { mine = post.WaitOne(0); }
                    catch (System.Threading.AbandonedMutexException) { mine = true; } // l'ancien est mort : le poste est libre
                    if (!mine) return 0;
                }
                Prepare();
                if (File.Exists(Log) && new FileInfo(Log).Length > 256 * 1024) File.Delete(Log);
                Application.Run(new Passeur(args[0]));
                post.ReleaseMutex();
            }
            return 0;
        }

        /// <summary>Un vol d'essai : le sprite d'un fichier, de la session (ou du processus) `from` à `to`.</summary>
        public static string Try(string spriteFile, string from, string to, int cols, int rows)
        {
            Prepare();
            var sprite = Sprite.From(new JavaScriptSerializer { MaxJsonLength = int.MaxValue }.DeserializeObject(File.ReadAllText(spriteFile)));
            if (sprite == null) return "sprite illisible : " + spriteFile;
            var scene = new Scene { Cols = cols, Rows = rows };
            uint a = PidOf(from), b = PidOf(to);
            if (a == 0 || b == 0) return "session introuvable : " + (a == 0 ? from : to);
            Place pa = Places.Of(a, scene), pb = Places.Of(b, scene);
            if (pa == null || pb == null) return "pas de fenêtre visible pour " + (pa == null ? from : to);
            if (pa.Window == pb.Window) return "les deux sessions partagent une fenêtre (des onglets)";
            if (!pa.InSight() || !pb.InSight()) return "la fenêtre de " + (pa.InSight() ? to : from) + " est cachée par une autre";
            var flight = new Flight(sprite, pa, pb);
            flight.FormClosed += delegate { Application.ExitThread(); };
            flight.Show();
            Application.Run();
            return string.Format("vol de {0} ({1:0},{2:0}) à {3} ({4:0},{5:0}), cellules {6}x{7} puis {8}x{9}",
                from, pa.Right, pa.Top, to, pb.Right, pb.Top, pa.CellW, pa.CellH, pb.CellW, pb.CellH);
        }

        static void Prepare()
        {
            Win.SetProcessDpiAwarenessContext(new IntPtr(-4)); // par écran : les positions en vrais pixels, chaque écran à son échelle
        }

        Passeur(string source)
        {
            this.source = source;
            sourceTime = File.GetLastWriteTimeUtc(source);
            storeFile = NewestStore();
            timer.Tick += delegate
            {
                try { Tick(); }
                catch (Exception e) { Note("erreur : " + e.Message); }
            };
            timer.Start();
            Note("le passeur prend son poste");
        }

        void Tick()
        {
            Poll();
            if (++ticks % 40 != 0) return; // toutes les dix secondes
            if (File.GetLastWriteTimeUtc(source) != sourceTime) Leave("son code a changé : le prochain lancement prendra le nouveau");
            storeFile = NewestStore(); // le stockage d'un plugin réinstallé, sous un autre nom
            // plus aucune session ouverte depuis une minute : il rentre
            lonely = LiveSessions() == 0 ? lonely + 1 : 0;
            if (lonely >= 6) Leave("plus aucune session");
        }

        void Leave(string why)
        {
            Note("le passeur s'en va : " + why);
            timer.Stop();
            ExitThread();
        }

        void Poll()
        {
            if (storeFile == null) return;
            DateTime time;
            try { time = File.GetLastWriteTimeUtc(storeFile); } catch { return; }
            if (time == storeTime) return;
            string text;
            try
            {
                using (var stream = new FileStream(storeFile, FileMode.Open, FileAccess.Read, FileShare.ReadWrite | FileShare.Delete))
                using (var reader = new StreamReader(stream, Encoding.UTF8)) text = reader.ReadToEnd();
            }
            catch
            {
                return; // pris par un écrivain : au prochain coup d'œil
            }
            var raw = Members(text, "visit:");
            if (raw == null) return; // à moitié écrit : au prochain coup d'œil
            storeTime = time;
            var now = new Dictionary<string, Seen>();
            foreach (var kv in raw)
            {
                string id = kv.Key.Substring(6);
                Seen was;
                if (seen.TryGetValue(id, out was) && was.Raw == kv.Value) { now[id] = was; continue; } // inchangée : rien à décoder
                var v = json.DeserializeObject(kv.Value) as Dictionary<string, object>;
                if (v != null) now[id] = new Seen { Raw = kv.Value, Host = Json.Str(v, "host"), Phase = Json.Str(v, "phase") ?? "there", Visit = v };
            }
            if (seeded)
            {
                foreach (var kv in now)
                {
                    Seen was;
                    seen.TryGetValue(kv.Key, out was);
                    // il arrive chez son hôte : de chez lui à chez l'hôte
                    if (kv.Value.Phase == "there" && (was == null || was.Host != kv.Value.Host)) Fly(kv.Key, kv.Value, text, true);
                    // l'hôte l'a raccompagné : de chez l'hôte à chez lui
                    else if (kv.Value.Phase == "gone" && was != null && was.Phase != "gone") Fly(kv.Key, kv.Value, text, false);
                }
                // parti sans que l'hôte ait répondu (il rentre de lui-même au bout de neuf secondes)
                foreach (var kv in seen)
                    if (!now.ContainsKey(kv.Key) && kv.Value.Phase == "leaving") Fly(kv.Key, kv.Value, text, false);
            }
            seen = now;
            seeded = true;
        }

        /// <summary>
        /// Les membres du premier niveau dont la clé commence par `prefix` : la clé, et le texte JSON de
        /// sa valeur, sans rien décoder ; null si le texte est incomplet (lu pendant qu'on l'écrivait).
        /// </summary>
        public static Dictionary<string, string> Members(string text, string prefix)
        {
            var found = new Dictionary<string, string>();
            int depth = 0, from = -1, n = text.Length;
            string key = null;
            for (int i = 0; i < n; i++)
            {
                char c = text[i];
                if (c == '"')
                {
                    int start = ++i;
                    while (i < n && text[i] != '"') { if (text[i] == '\\') i++; i++; }
                    if (depth == 1 && key == null)
                    {
                        int j = i + 1;
                        while (j < n && char.IsWhiteSpace(text[j])) j++;
                        if (j < n && text[j] == ':')
                        {
                            if (i - start >= prefix.Length && string.CompareOrdinal(text, start, prefix, 0, prefix.Length) == 0) { key = text.Substring(start, i - start); from = j + 1; }
                            i = j;
                        }
                    }
                    continue;
                }
                if (c == '{' || c == '[') depth++;
                else if (c == '}' || c == ']')
                {
                    depth--;
                    if (key != null && depth == 1) { found[key] = text.Substring(from, i + 1 - from); key = null; }
                    else if (key != null && depth == 0) { found[key] = text.Substring(from, i - from).Trim(); key = null; }
                }
                else if (c == ',' && depth == 1 && key != null) { found[key] = text.Substring(from, i - from).Trim(); key = null; }
            }
            return depth == 0 && text.TrimEnd().EndsWith("}") ? found : null;
        }

        void Fly(string visitor, Seen visit, string text, bool outward)
        {
            string who = Json.Str(visit.Visit, "name") ?? visitor, host = visit.Host;
            var sprite = Sprite.From(Json.Get(visit.Visit, "sprite"));
            if (sprite == null) { Note(who + (outward ? " part" : " rentre") + " sans voler : pas de sprite (une session d'avant le passeur)"); return; }
            // la scène de chacun telle qu'il la dit à tous, décodée ici seulement ; celle du visiteur à son départ, à défaut
            Dictionary<string, object> presence = null;
            var members = Members(text, "presence");
            string part;
            if (members != null && members.TryGetValue("presence", out part)) presence = json.DeserializeObject(part) as Dictionary<string, object>;
            Scene home = SceneOf(presence, visitor) ?? Scene.From(Json.Get(visit.Visit, "scene"));
            Scene away = SceneOf(presence, host);
            uint homePid = PidOf(visitor), hostPid = PidOf(host);
            string why = homePid == 0 ? "sa session est introuvable"
                : hostPid == 0 ? "la session de l'hôte est introuvable"
                : home == null ? "sa scène n'est pas affichée"
                : away == null ? "la scène de l'hôte n'est pas affichée, ou il tourne sur un mod d'avant le passeur"
                : null;
            Place a = why == null ? Places.Of(homePid, home) : null, b = why == null ? Places.Of(hostPid, away) : null;
            if (why == null && a == null) why = "sa session n'a pas de fenêtre visible (de fond, ou réduite)";
            if (why == null && b == null) why = "la session de l'hôte n'a pas de fenêtre visible (de fond, ou réduite)";
            if (why == null && a.Window == b.Window) why = "les deux sessions partagent une fenêtre (des onglets)";
            if (why == null && !a.InSight()) why = "sa fenêtre est cachée par une autre";
            if (why == null && !b.InSight()) why = "la fenêtre de l'hôte est cachée par une autre";
            if (why != null) { Note(who + (outward ? " part chez " : " rentre de chez ") + host + " sans voler : " + why); return; }
            Note(who + (outward ? " s'envole chez " : " rentre de chez ") + host);
            (outward ? new Flight(sprite, a, b) : new Flight(sprite, b, a)).Show();
        }

        static Scene SceneOf(Dictionary<string, object> presence, string session)
        {
            return Scene.From(Json.Get(Json.Get(presence, session) as Dictionary<string, object>, "scene"));
        }

        static string NewestStore()
        {
            string dir = Path.Combine(Home, "plugins", "store");
            if (!Directory.Exists(dir)) return null;
            string best = null;
            DateTime bestTime = DateTime.MinValue;
            foreach (var f in Directory.GetFiles(dir, "diorama*.json"))
            {
                var t = File.GetLastWriteTimeUtc(f);
                if (t > bestTime) { best = f; bestTime = t; }
            }
            return best;
        }

        /// <summary>Le processus d'une session (ou un numéro de processus tel quel), s'il vit ; 0 sinon.</summary>
        static uint PidOf(string session)
        {
            uint pid;
            if (uint.TryParse(session, out pid)) return Alive(pid) ? pid : 0;
            string dir = Path.Combine(Home, "sessions");
            if (!Directory.Exists(dir)) return 0;
            var reader = new JavaScriptSerializer();
            foreach (var f in Directory.GetFiles(dir, "*.json"))
            {
                // le fichier porte le numéro du processus : on ne lit que ceux qui vivent
                if (!uint.TryParse(Path.GetFileNameWithoutExtension(f), out pid) || !Alive(pid)) continue;
                try
                {
                    var d = reader.DeserializeObject(File.ReadAllText(f)) as Dictionary<string, object>;
                    if (Json.Str(d, "sessionId") == session) return pid;
                }
                catch { }
            }
            return 0;
        }

        /// <summary>Combien de sessions vivent : par le nom de leur fichier, sans en lire aucun.</summary>
        static int LiveSessions()
        {
            string dir = Path.Combine(Home, "sessions");
            if (!Directory.Exists(dir)) return 0;
            int n = 0;
            uint pid;
            foreach (var f in Directory.GetFiles(dir, "*.json"))
                if (uint.TryParse(Path.GetFileNameWithoutExtension(f), out pid) && Alive(pid)) n++;
            return n;
        }

        /// <summary>Si le processus vit encore, sans garder de poignée sur lui.</summary>
        static bool Alive(uint pid)
        {
            if (pid == 0) return false;
            IntPtr process = Win.OpenProcess(0x1000, false, pid); // PROCESS_QUERY_LIMITED_INFORMATION
            if (process == IntPtr.Zero) return false;
            uint code;
            bool ok = Win.GetExitCodeProcess(process, out code);
            Win.CloseHandle(process);
            return ok && code == 259; // STILL_ACTIVE
        }

        static void Note(string line)
        {
            try { File.AppendAllText(Log, DateTime.Now.ToString("yyyy-MM-dd HH:mm:ss ") + line + Environment.NewLine); } catch { }
        }
    }
}
