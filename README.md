# music_sync

Simple tool to scan a music collection and find missing albums.

## usage

```bash
Usage: music_sync [OPTIONS] <FOLDER>

Arguments:
  <FOLDER>  Collection root folder

Options:
  -a, --artist <ARTIST>          Artist filter (can be specified multiple times)
  -r, --rate-limit <RATE_LIMIT>  MusicBrainz API calls per second, default 1
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

## how it works

This application relies on MusicBrainz APIs. First of all, it scans your root folder using folder names as artist names, and makes a single MusicBrainz search query for all artists to retrieve artist ids, then scans every artist subfolder, making a single MusicBrainz query for all existing artist albums, to translate found names into correct names. After that, it queries MusicBrainz again for artist's full album list, excludes from that list albums found on previous query, and prints the missing albums list.
