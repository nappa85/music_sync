# music_sync

Simple tool to scan a music collection and find missing albums.

## usage

```bash
Usage: music_sync [OPTIONS] <FOLDER>

Arguments:
  <FOLDER>  

Options:
  -a, --artist <ARTIST>  
  -h, --help             Print help
  -V, --version          Print version
```

where `folder` is the root of the music collection, subfolders are artist names, sub-subfolders are albums prefixed with publication year.

```bash
$ music_sync ./music
Missing album ./music/Katatonia/1996 - Brave Murder Day
Missing album ./music/Pink Floyd/1979 - The Wall
Missing album ./music/Lez Zeppelin/1970 - Led Zeppelin III
$ music_sync -a katatonia ./music
Missing album ./music/Katatonia/1996 - Brave Murder Day
```
