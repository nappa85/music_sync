# music_sync

Simple tool to scan a music collection and find missing albums.

## usage

```bash
Usage: music_sync [OPTIONS] <FOLDER>

Arguments:
  <FOLDER>  

Options:
  -a, --artist <ARTIST>          
  -r, --rate-limit <RATE_LIMIT>  
  -h, --help                     Print help
  -V, --version                  Print version
```

where `folder` is the root of the music collection, subfolders are artist names, sub-subfolders are albums prefixed with publication year.

```bash
$ music_sync ./music
Missing album ./music/Katatonia/1996 - Brave Murder Day
Missing album ./music/Pink Floyd/1979 - The Wall
Missing album ./music/Lez Zeppelin/1970 - Led Zeppelin III
```

`artist` parameter is useful to operate on a single artist at time

```bash
$ music_sync -a katatonia ./music
Missing album ./music/Katatonia/1996 - Brave Murder Day
```

`rate_limit` parameter can be used to override the default rate limit of 1 request per second. Keep in mind that sending more than 1 request per second could end up with your IP being banned from MusicBrainz servers, to know more about rate limit read [the docs](https://musicbrainz.org/doc/MusicBrainz_API/Rate_Limiting).
