// CollectionDowngrader.LazerSchema from https://github.com/ookiineko/CollectionDowngrader/tree/main/LazerSchema
using RealmHashExtractor.LazerSchema;
using CommandLine;
using Realms;
using System.Text.Json;
using System.Data;

string? outputFile = null;
string? realmFile = null;
List<string> user_collections = [];
Parser.Default.ParseArguments<CliOptions>(args)
    .WithParsed<CliOptions>(o =>
    {
        realmFile = o.RealmFile;
        outputFile = o.Output;
        if (o.Collections != null)
        {
            user_collections = o.Collections.ToList();
        }
    });

if (!File.Exists(realmFile))
{
    Console.WriteLine($"File not found: {realmFile}");
    return;
}

const int LazerSchemaVersion = 44;
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
if (user_collections.Count > 0)
{
    collections = collections.Where(c => user_collections.Contains(c.Name)).ToList();
}
Console.Error.WriteLine($"Loaded {collections.Count} collections");

Dictionary<(string, string?), List<BeatmapCleanMetadata>> beatmapsByAudioBGFiles = [];

foreach (BeatmapCollection collection in collections)
{
    Console.Error.WriteLine($"Collection: {collection.Name}, with {collection.BeatmapMD5Hashes.Count} difficulties");
    foreach (string hash in collection.BeatmapMD5Hashes)
    {
        // Search for the beatmap with this hash
        var beatmaps = db.All<Beatmap>().Where(b => b.MD5Hash == hash).ToList();
        foreach (Beatmap beatmap in beatmaps)
        {
            if (beatmap.BeatmapSet == null)
            {
                continue;
            }
            // The file is case-insensitive.
            var audioName = beatmap.Metadata.AudioFile;
            string? audioHash = null;
            var bgName = beatmap.Metadata.BackgroundFile;
            string? bgHash = null;
            var files = beatmap.BeatmapSet.Files;
            foreach (RealmNamedFileUsage file in files)
            {
                if (string.Equals(file.Filename, audioName, StringComparison.OrdinalIgnoreCase))
                {
                    audioHash = file.File.Hash;
                }
                if (string.Equals(file.Filename, bgName, StringComparison.OrdinalIgnoreCase))
                {
                    bgHash = file.File.Hash;
                }
            }
            if (audioHash == null || bgHash == null)
            {
                Console.Error.WriteLine($"      Missing audio or bg hash for {beatmap.Metadata.Artist} - {beatmap.Metadata.Title} [{beatmap.DifficultyName}] {beatmap.MD5Hash}");
                // Console.Error.WriteLine($"       audioName: {audioName}, bgName: {bgName}");
                // Console.Error.WriteLine($"       Files: {string.Join(", ", files)}");
                if (audioHash == null)
                    continue;
                Console.Error.WriteLine("       (Continue as audio exists)");
            }
            var metadata = new BeatmapCleanMetadata
            {
                Title = beatmap.Metadata.Title,
                TitleUnicode = beatmap.Metadata.TitleUnicode,
                Artist = beatmap.Metadata.Artist,
                ArtistUnicode = beatmap.Metadata.ArtistUnicode,
                Source = beatmap.Metadata.Source,
            };
            if (beatmap.Metadata.Tags != null)
            {
                metadata.Tags = [.. beatmap.Metadata.Tags.Split(" ")];
            }
            if (!beatmapsByAudioBGFiles.TryGetValue((audioHash, bgHash), out List<BeatmapCleanMetadata>? v)) {
                beatmapsByAudioBGFiles[(audioHash, bgHash)] = [metadata];
            } else {
                v.Add(metadata);
            }
        }
    }
}

// audioHash -> Metadata
Dictionary<string, BeatmapFileMetadataInfo> beatmapFileMetadataInfos = [];

foreach (var pair in beatmapsByAudioBGFiles)
{
    var audioHash = pair.Key.Item1;
    var bgHash = pair.Key.Item2;
    var metadatas = pair.Value;
    if (!beatmapFileMetadataInfos.TryGetValue(audioHash, out BeatmapFileMetadataInfo value))
    {
        beatmapFileMetadataInfos[audioHash] = new BeatmapFileMetadataInfo(audioHash, bgHash, metadatas);
    }
    else
    {
        value.Update(bgHash, metadatas);
    }
}

// Duplicate checking
Dictionary<(string, string), int> titleArtistSet = [];
foreach (var pair in beatmapFileMetadataInfos.Values) {
    var title = pair.Metadata.Title;
    var artist = pair.Metadata.Artist;
    var key = (title, artist);

    if (titleArtistSet.TryGetValue(key, out int currentCount)) {
        titleArtistSet[key] = currentCount + 1;
    } else {
        titleArtistSet[key] = 1;
    }
}

foreach (var item in titleArtistSet) {
    if (item.Value > 1) {
        Console.Error.WriteLine($"Duplicate found: Title = {item.Key.Item1}, Artist = {item.Key.Item2}, Count = {item.Value}");
    }
}

string jsonString = JsonSerializer.Serialize(beatmapFileMetadataInfos.Values);
if (outputFile == null)
{
    Console.WriteLine(jsonString);
}
else
{
    File.WriteAllText(outputFile, jsonString);
}


class CliOptions
{
    [Option('c', "collection", Required = false, HelpText = "Collection name. If not provided, all collections will be processed. Songs not in any collection will be ignored.")]
    public IEnumerable<string>? Collections { get; set; }

    [Value(0, MetaName = "RealmFile", Required = true, HelpText = "Path to realm file")]
    public required string RealmFile { get; set; }

    [Option('o', "output", Required = false, HelpText = "File to output JSON. Stdout if not provided.")]
    public string? Output { get; set; }
}

struct BeatmapCleanMetadata
{
    public string Title { get; set; }
    public string TitleUnicode { get; set; }
    public string Artist { get; set; }
    public string ArtistUnicode { get; set; }
    public string Source { get; set; }
    public HashSet<string> Tags { get; set; }

    public override readonly int GetHashCode()
    {
        // Don't include tags
        return HashCode.Combine(Title, TitleUnicode, Artist, ArtistUnicode, Source);
    }

    public override readonly string ToString()
    {
        return $"Title: {Title}, TitleUnicode: {TitleUnicode}, Artist: {Artist}, ArtistUnicode: {ArtistUnicode}, Source: {Source}, Tags: {Tags}";
    }

    public void InplaceMerge(BeatmapCleanMetadata others)
    {
        if (Title.Length == 0)
        {
            Title = others.Title;
        }
        else if (Title != others.Title)
        {
            Console.Error.WriteLine($"Different title when merging: {Title} <- {others.Title}");
        }
        if (TitleUnicode.Length == 0)
        {
            TitleUnicode = others.TitleUnicode;
        }
        else if (TitleUnicode != others.TitleUnicode)
        {
            Console.Error.WriteLine($"Different title (unicode) when merging: {TitleUnicode} <- {others.TitleUnicode}");
        }
        if (Artist.Length == 0)
        {
            Artist = others.Artist;
        }
        else if (Artist != others.Artist)
        {
            Console.Error.WriteLine($"Different artist when merging: {Artist} <- {others.Artist}");
        }
        if (ArtistUnicode.Length == 0)
        {
            ArtistUnicode = others.ArtistUnicode;
        }
        else if (ArtistUnicode != others.ArtistUnicode)
        {
            Console.Error.WriteLine($"Different artist (unicode) when merging: {ArtistUnicode} <- {others.ArtistUnicode}");
        }
        if (Source.Length == 0)
        {
            Source = others.Source;
        }
        else if (Source != others.Source)
        {
            Console.Error.WriteLine($"Different source when merging: {Source} <- {others.Source}");
        }
        Tags.UnionWith(others.Tags);
    }
}

struct BeatmapFileMetadataInfo
{
    public string AudioHash { get; set; }
    public HashSet<string?> BGHashes { get; set; }
    public BeatmapCleanMetadata Metadata { get; set; }

    public BeatmapFileMetadataInfo(string audioHash, string? bgHash, List<BeatmapCleanMetadata> metadatas)
    {
        AudioHash = audioHash;
        if (bgHash != null) {
            BGHashes = [bgHash];
        } else {
            BGHashes = [];
        }
        if (metadatas.Count == 0)
        {
            throw new ArgumentException("Empty metadata list");
        }
        BeatmapCleanMetadata metadata = metadatas[0];
        for (int i = 1; i < metadatas.Count; i++)
        {
            metadata.InplaceMerge(metadatas[i]);
        }
        Metadata = metadata;
    }

    public readonly void Update(string? bgHash, List<BeatmapCleanMetadata> metadatas)
    {
        if (bgHash != null)
        {
            BGHashes.Add(bgHash);
        }
        if (metadatas.Count == 0)
        {
            throw new ArgumentException("Empty metadata list");
        }
        BeatmapCleanMetadata metadata = metadatas[0];
        for (int i = 1; i < metadatas.Count; i++)
        {
            metadata.InplaceMerge(metadatas[i]);
        }
        Metadata.InplaceMerge(metadata);
    }
}
