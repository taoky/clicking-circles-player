// CollectionDowngrader.LazerSchema from https://github.com/ookiineko/CollectionDowngrader/tree/main/LazerSchema
using CollectionDowngrader.LazerSchema;
using Realms;
using System.Text.Json;

if (args.Length < 1)
{
    Console.WriteLine("Please provide path to realm file as argument");
    return;
}

string realmFile = Path.GetFullPath(args[0]);
if (!File.Exists(realmFile))
{
    Console.WriteLine($"File not found: {realmFile}");
    return;
}

const int LazerSchemaVersion = 41;
RealmConfiguration config = new(realmFile)
{
    IsReadOnly = true,
    SchemaVersion = LazerSchemaVersion,
    Schema = new[] {
    typeof(Beatmap),
    typeof(BeatmapCollection),
    typeof(BeatmapDifficulty),
    typeof(BeatmapMetadata),
    typeof(BeatmapSet),
    typeof(BeatmapUserSettings),
    typeof(RealmFile),
    typeof(RealmNamedFileUsage),
    typeof(RealmUser),
    typeof(Ruleset),
    typeof(ModPreset)
}
};

Realm db = Realm.GetInstance(config);
List<BeatmapCollection> collections = [.. db.All<BeatmapCollection>()];
Console.Error.WriteLine($"Loaded {collections.Count} collections");

Dictionary<(string, string), BeatmapCleanMetadata> beatmapsByAudioBGFiles = [];

foreach (BeatmapCollection collection in collections)
{
    Console.Error.WriteLine($"Collection: {collection.Name}, with {collection.BeatmapMD5Hashes.Count} difficulties");
    foreach (string hash in collection.BeatmapMD5Hashes)
    {
        // Console.WriteLine($"  {hash}");
        // Search for the beatmap with this hash
        var beatmaps = db.All<Beatmap>().Where(b => b.MD5Hash == hash).ToList();
        foreach (Beatmap beatmap in beatmaps)
        {
            if (beatmap.BeatmapSet == null)
            {
                continue;
            }
            // Console.WriteLine($"    Beatmap: {beatmap.Metadata.Artist} - {beatmap.Metadata.Title} [{beatmap.DifficultyName}] {beatmap.MD5Hash}");
            // Console.WriteLine($"      Audio: {beatmap.Metadata.AudioFile}, BG: {beatmap.Metadata.BackgroundFile}");
            var audioName = beatmap.Metadata.AudioFile;
            string? audioHash = null;
            var bgName = beatmap.Metadata.BackgroundFile;
            string? bgHash = null;
            var files = beatmap.BeatmapSet.Files;
            foreach (RealmNamedFileUsage file in files)
            {
                if (file.Filename == audioName)
                {
                    // Console.WriteLine($"      Audio: {file.Filename} ({file.File.Hash})");
                    audioHash = file.File.Hash;
                }
                if (file.Filename == bgName)
                {
                    // Console.WriteLine($"      BG: {file.Filename} ({file.File.Hash})");
                    bgHash = file.File.Hash;
                }
                // Console.WriteLine($"      File: {file.Filename} ({file.File.Hash})");
            }
            if (audioHash == null || bgHash == null)
            {
                Console.Error.WriteLine($"      Missing audio or bg hash for {beatmap.Metadata.Artist} - {beatmap.Metadata.Title} [{beatmap.DifficultyName}] {beatmap.MD5Hash}");
                continue;
            }
            var metadata = new BeatmapCleanMetadata
            {
                Title = beatmap.Metadata.Title,
                TitleUnicode = beatmap.Metadata.TitleUnicode,
                Artist = beatmap.Metadata.Artist,
                ArtistUnicode = beatmap.Metadata.ArtistUnicode,
                Source = beatmap.Metadata.Source,
                Tags = beatmap.Metadata.Tags
            };
            if (beatmapsByAudioBGFiles.ContainsKey((audioHash, bgHash)) && !beatmapsByAudioBGFiles[(audioHash, bgHash)].Equals(metadata))
            {
                Console.Error.WriteLine($"      Duplicate audio and bg hash for {beatmap.Metadata.Artist} - {beatmap.Metadata.Title} [{beatmap.DifficultyName}] {beatmap.MD5Hash}");
                Console.Error.WriteLine($"        Old: {beatmapsByAudioBGFiles[(audioHash, bgHash)]}");
                Console.Error.WriteLine($"        New: {metadata}");
                Console.Error.WriteLine("        Overwriting anyway.");
            }
            beatmapsByAudioBGFiles[(audioHash, bgHash)] = metadata;
        }
    }
}

// audioHash -> Metadata
Dictionary<string, BeatmapFileMetadataInfo> beatmapFileMetadataInfos = [];

foreach (var pair in beatmapsByAudioBGFiles)
{
    // var (audioHash, bgHash) = pair.Key;
    // var beatmaps = pair.Value;
    // Console.WriteLine($"{beatmaps.Artist} - {beatmaps.Title}");
    // Console.WriteLine($"  Audio: {audioHash}, BG: {bgHash}");
    var audioHash = pair.Key.Item1;
    var bgHash = pair.Key.Item2;
    var metadata = pair.Value;
    if (!beatmapFileMetadataInfos.TryGetValue(audioHash, out BeatmapFileMetadataInfo value))
    {
        beatmapFileMetadataInfos[audioHash] = new BeatmapFileMetadataInfo
        {
            AudioHash = audioHash,
            BGHashes = [bgHash],
            Metadata = metadata
        };
    }
    else
    {
        value.BGHashes.Add(bgHash);
        if (!value.Metadata.Equals(metadata))
        {
            Console.Error.WriteLine($"      Different metadata for {metadata.Artist} - {metadata.Title}");
            Console.Error.WriteLine($"        Old: {value.Metadata}");
            Console.Error.WriteLine($"        New: {metadata}");
        }
    }
}

string jsonString = JsonSerializer.Serialize(beatmapFileMetadataInfos.Values);
Console.WriteLine(jsonString);


struct BeatmapCleanMetadata
{
    public string Title { get; set; }
    public string TitleUnicode { get; set; }
    public string Artist { get; set; }
    public string ArtistUnicode { get; set; }
    public string Source { get; set; }
    public string? Tags { get; set; }

    public override readonly bool Equals(object? obj)
    {
        if (obj is not BeatmapCleanMetadata other)
        {
            return false;
        }

        // Don't compare tags
        return Title == other.Title &&
            TitleUnicode == other.TitleUnicode &&
            Artist == other.Artist &&
            ArtistUnicode == other.ArtistUnicode &&
            Source == other.Source;
    }

    public override readonly int GetHashCode()
    {
        // Don't include tags
        return HashCode.Combine(Title, TitleUnicode, Artist, ArtistUnicode, Source);
    }

    public override readonly string ToString()
    {
        return $"Title: {Title}, TitleUnicode: {TitleUnicode}, Artist: {Artist}, ArtistUnicode: {ArtistUnicode}, Source: {Source}, Tags: {Tags}";
    }
}

struct BeatmapFileMetadataInfo
{
    public string AudioHash { get; set; }
    public List<string> BGHashes { get; set; }
    public BeatmapCleanMetadata Metadata { get; set; }
}
