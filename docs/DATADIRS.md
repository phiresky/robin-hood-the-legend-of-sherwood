This document lists ALL KNOWN versions of this game and how they differ.

# Language Codes

```
1028 - Chinese (Taiwan)
1029 - Czech +
1031 - German (Germany) +
1033 - English (US)
1036 - French (France) +
1040 - Italian (Italy) +
1041 - Japanese +
1042 - Korean
1045 - Polish +
1046 - Portuguese (Brazil)
1049 - Russian +
1054 - Thai
2047 - English (GB) +
2052 - Chinese (PRC)
2070 - Portuguese (Portugal)
3082 - Spanish (Spain Modern Sort) +
```

# Leicester Demo — "The Scarlet Night"

Mission: Rescue Will Scarlet from the Leicester jail. All known builds play the same
mission but differ in tutorials, Level.res size, and other details.

## Windows — ECoste build (2002-09-18, with tutorials)

Compiled by developer `ECoste` on their Windows machine. PE timestamp: 2002-09-18.

English (1033):

    filename: setup_demo_ecoste.exe (originally setup_us_demo.exe / Setup_us.exe / Robin_Hood_LS_Demo_1_an.exe)
    format:   Wise Installer (80.8 MB)
    source:   https://www.moddb.com/games/robin-hood-the-legend-of-sherwood/downloads/robin-hood-the-legend-of-sherwood-demo-robin-hood-the-legend-of-sherwood
    source:   https://archive.org/details/RobinHoodTheLegendOfSherwoodDemo
    source:   https://archive.org/details/Robin_Hood_demo (as "RH demo.exe" in zip)
    source:   http://quedzasvideogames.free.fr/robin_hood/telechargements_rh.php
    MD5:      a8c4df5cbf009f3381ba582e6fe6c5f2
    SHA256:   68cca8fd84ac87bac22f7092fd69282986f25107f43110c80726f34e3dc8c9ec

French (1036):

    filename: Robin_Hood_LS_Demo_1_fr.exe
    format:   Wise Installer (84.2 MB)
    source:   http://quedzasvideogames.free.fr/robin_hood/telechargements_rh.php
    MD5:      04e6e5e4f5edec9d2baccd909f3db8bf
    SHA256:   69ade643e7c5f700c1f4af1858dccd4f79dd51a87f1c742f573b59f86971a9f8

German (1031):

    filename: Robin_Hood_LS_Demo_1_al.exe (also Setup_de.exe on GameStar disc)
    format:   Wise Installer (81.0 MB)
    source:   http://quedzasvideogames.free.fr/robin_hood/telechargements_rh.php
    source:   https://archive.org/details/gscda2002 (GameStar 12/2002 disc, Demos/Robin/Setup_de.exe)

Also on PC Games 01/2003 DVD (`PCGDVD0103`) as `Demos/RobinHood/Setup_PCGames.exe` (112.8 MB —
larger than other demo builds, possibly a combined or extended demo). ISO incomplete on
archive.org, file could not be extracted to verify.
    MD5:      d6ca71596f36d686c7ff00ad5c9c24e1
    SHA256:   ab7bd1312fca4b1dfcea0e1ead48cceccbe03bdd360624ab9deaff13f9917c5d

- Locale: 1033 (English US) / 1036 (French) / 1031 (German)
- SCB script: 52 classes, includes 10 `Tutorial_*` classes for in-game hints
  - Character tutorials: Robin, Marianne, Tuck, Little John (Petit_jean), beggar (mendiant)
  - Mechanic tutorials: drawbridge (PontLevis 1/2/3), lockpicking (Crocheter), herbs (Trefle)
  - Each tutorial triggers `DisplayPopupText` when a PC actor interacts with a parchment object
- Level.res: 41 resources — only Leicester mission text + tutorial strings + mission pictures
- UI string table (343 strings) includes "3D sound" option (inserted at index 53)
- `ParchmentPrison` script shows 4 popup texts (IDs 10, 11, 23, 25)

This is the version our existing `datadirs/demo` matches (identical file hashes).

## Windows — Pariso build (2002-08-19, no tutorials)

Same mission, but an earlier build without the tutorial system.
Compiled by developer `PARISO~1.ROO` (Pariso?) on their Windows machine. PE timestamp: 2002-08-19.

    filename: setup_demo_pariso.exe (originally setup_english.exe / Setup_Demo_English.exe)
    format:   Wise Installer (83 MB)
    source:   https://www.moddb.com/games/robin-hood-the-legend-of-sherwood/downloads/robin-hood-the-legend-of-sherwood-demo
    MD5:      27b181a7ad61748447834e7deecdae94
    SHA256:   52b69cabad7757612283c5b3df5a4e79eb11d88538424837ca6dc0a8704dbf8d

- Locale: 1033 (English US)
- SCB script: 42 classes — no tutorial classes at all
- Level.res: 508 resources — accidentally includes text/pictures for ALL full game missions
  (mission titles, briefings, popup texts, wave paths for every level), not just Leicester
- UI string table (343 strings) lacks "3D sound" option, shifted by -1 relative to ECoste build
- `ParchmentPrison` script shows only 2 popup texts (IDs 10, 11)
- Title.bfn is 82 KB vs ECoste build's 56 KB (more glyphs)

## Windows — Differences between the two builds

Both play the same Leicester mission and produce identical file trees (same filenames),
but 17 files differ:

| File | ECoste build | Pariso build |
|------|----------|---------------|
| `Robin Hood.exe` | 2,891,776 bytes | 2,842,624 bytes |
| `Dem_Lei_MP.scb` | 64,632 bytes (52 classes) | 59,640 bytes (42 classes) |
| `Dem_Lei_MP.rhm` | 15,669 bytes | 15,149 bytes |
| `leicester.rhp` | same size, different content | same size, different content |
| `Level.res` | 927 KB (41 resources) | 3.5 MB (508 resources) |
| `Slideshow_in.pak` | 122 KB (3 images: Wanadoo, Strategy First, Spellbound) | 82 KB (2 images: Wanadoo, Spellbound — no Strategy First logo) |
| `DEFAULT.RES` | has extra cursor | slightly smaller |
| `Title.bfn` | 56 KB | 82 KB |
| 3 other `.bfn` fonts | slightly different sizes | slightly different sizes |
| `keyset1/2.cfg` | fewer bindings | more bindings |
| 3 `.wav` files | slightly different voice lines | slightly different voice lines |

Both include `Fmod.dll` (identical), `WiseUpdt.exe`, and DirectX 8.1 redistributables in `TEMP/`.

## Linux — Runesoft port

Same game data as the ECoste build above. Ported by Runesoft.

    filename: rh-linux-demo-x86.run
    format:   Makeself 2.1.3 self-extracting archive (~62 MiB)
    source:   https://archive.org/details/rh-linux-demo-x86
    MD5:      b98a9b4abe44787b2c05443fb350dd2e
    SHA256:   05a63fe131741d351ece9040d0fd3e0a452d18232ebaab183b0a68030e4fcb5e

Extract without executing:

    OFFSET=$(head -n 361 rh-linux-demo-x86.run | wc -c)
    tail -c +$((OFFSET + 1)) rh-linux-demo-x86.run | gunzip | tar x

Produces a `robinhood_demo/` directory with `Data/`, `1033/`, `arial.ttf`, and `robin_demo` (32-bit x86 ELF binary).

## AmigaOS 4 / MorphOS — PowerPC port

Same game data as the ECoste build above. Port for AmigaOS 4 / MorphOS (PowerPC).
Dated November 2006. Uses PowerSDL libraries. Includes `rh.png` icon not present in other versions.

    filename: RobinHood-demo.lha
    format:   LHA archive (63.9 MB)
    source:   https://www.morphos-storage.net/?id=1901771
    MD5:      02ed5fdaac6565ff3d71002699a2b745
    SHA256:   b3ad95b7255d1dce7f0b823df87503646d475cf2c5694577ad6bb4986b4eae4d

Binary: `robin_demo` — ELF 32-bit MSB relocatable, PowerPC.
Bundled libs: `powersdl.library`, `powersdl_gfx.library`, `powersdl_image.library`,
`powersdl_mixer.library`, `powersdl_net.library`, `powersdl_sound.library`, `powersdl_ttf.library`.

## Redump — German demo disc

| Field | Value |
|-------|-------|
| Redump | http://redump.org/disc/132050/ |
| Title | Robin Hood: Die Legende von Sherwood |
| Region | Germany |
| Language | German |
| Version | Demo v1.00 |
| Edition | Demo |
| Media | CD, Mode 1, 1 track |
| Size | 146,889,456 bytes |
| CRC-32 | `a90eeedc` |
| MD5 | `14dc71f72ca946d4b7707a71b6d25e55` |
| SHA-1 | `948dff7219681d2ee1d06cb745a3d29dea904bbd` |
| Mastering Code | ROBIN HOOD |
| Mastering SID | IFPI LTZ1 |

# Lincoln Demo — "Free Lincoln" (DEMO II)

Mission: Help Godwin recapture Lincoln castle.

English (1033):

    filename: Setup_demo_en_xp.exe
    format:   Wise Installer (92 MB)
    source:   https://archive.org/details/winxp-magazine-dvd-2003-06-issue-19-d
    MD5:      9e8edf0a4578d4a250f59abe010eda8e
    SHA256:   d058be90faaebdf06a7ff318927762c5e5ad19a53db0e01327972da4af5c9679

- Locale: 1033 (English US)
- EULA: English, Wanadoo Edition
- PE timestamp: 2002-11-19
- SCB script: `Demo_Lin.scb` (68,263 bytes, 58 classes, compiled by ECoste)
- Map: `lincoln.rhp` + `Lincoln.map`/`.min` (night)
- Includes `binkw32.dll`, `Loading.pak`; no `Slideshow_in.pak`
- 348 sound files, 6 music tracks (Lincoln-specific: `Lincoln_D.wav`, `Lincoln_NF.wav`)
- 46 character .rhs files

German (1031):

    filename: Robin_Hood_LS_Demo_2_al.exe (originally Setup_de.exe)
    format:   Wise Installer (74.6 MB)
    source:   https://web.archive.org/web/20110820155810/http://www.spellbound.de/files/demos/Robin_Hood/Demo2/Setup_de.exe
    source:   http://quedzasvideogames.free.fr/robin_hood/telechargements_rh.php
    MD5:      294e4e496d67a46077b7d8c7327efedb
    SHA256:   517704ff0e6e10e070cb1931d8f4303ce6c3bfa7a0683b5cfb917b3947381508

The fan site notes: *"Je n'ai jamais trouvé cette deuxième démonstration en une autre langue!"*
("I never found this second demo in any other language!") — however, an English build does exist (see above).

A developer data dump with both demo level sets also exists:

    filename: Levels.rar
    format:   RAR5 archive (12 MB)
    MD5:      508360551d0fa6003a694d78fbb9854d
    SHA256:   a56b7943c55068c9d4dc7206632edf74106cd76e0c5217d9f3e9174e86551a26

Contains `Demo_Lin.scb` (58 classes, compiled by ECoste, 2002-10-14), `Demo_Lin.rhm`,
`lincoln.rhp` (map geometry), `Lincoln.map`/`.min` (night map), plus the Leicester demo
files (`Dem_Lei_MP.*`, `leicester.rhp`, `Leicester.map`/`.min` — matching the ECoste build).

A YouTube video confirms this demo existed and was publicly released:
https://www.youtube.com/watch?v=LCXfCY0jx94
> "Two demo levels were released before the official game launch.
> This level was included as 'Free Lincoln' in the final release with some minor changes."

Reference notes:

- Version string: `DEMO II v1.00`
- Registry key: `Software/Spellbound Software/Robin Hood Demo II 1.0`
- Mission file: `Demo_Lin` (vs `Dem_Lei_MP` for Leicester)
- Proto-level/map: `Lincoln` (vs `Leicester`)
- Playable characters: `RSABC` — Robin, Scarlet, and 3 others (vs `RJMT` — Robin, Little John, Marian, Tuck for Leicester)
- Menu text table: resource ID `1000034` (vs `1000040` for Leicester)
- Uses `` (vs ``)
- Loading screen text: `"DEMO II v1.00"` (vs `"DEMO v1.00"`)

The Lincoln demo would have required different data files than the Leicester demo:
- A `Demo_Lin.scb` script (not `Dem_Lei_MP.scb`)
- A `Lincoln.rhp` map (not `leicester.rhp`)
- A `Level.res` with menu text at resource ID `1000034`

The English installer above contains all these files. The Pariso build's Level.res includes
full-game mission text for Lincoln (at resource `1000000`: "Free Lincoln" / "Godwin wants
to recapture his castle"), but it ships with Leicester map data and Leicester scripts — it
was never actually a Lincoln demo.

# Patch

    filename: patch_robin_hood.exe
    format:   Windows executable (2.9 MB)
    source:   http://quedzasvideogames.free.fr/robin_hood/telechargements_rh.php
    MD5:      94cd8ebeafdc6b324d938e291430bdef
    SHA256:   2aa97adf89383de880eccfb680bcda9b7aecfef38607e89b3b4bf02d838a485a

# Full Release

## Windows — Japanese retail (2003-03-14, v1.1)

Published by Imagineer Co., Ltd. (イマジニア株式会社), distributed by Capcom (株式会社カプコン).
Developer: Spellbound Entertainment AG. Flip-top big box, ¥7,980. Version 1.1.

    filename: RBNH203801.iso
    format:   CD image (637 MB)
    source:   https://archive.org/details/robin-hood-the-legend-of-sherwood-jp-20030314-win
    MD5:      a19342b1b22e0de7f29fb02bdedbaeb7
    SHA256:   c9a431a8862bad906529df3eb2e71f2c600b2dcaef74db3a72068674c634a080

- Locale: 1041 (Japanese)
- A v1.15 update patch is also available at the same archive.org item (Update_1.15.7z, 5.4 MB)

## ~~Windows — Czech magazine coverdisc (Score #152, October 2006)~~ NOT THIS GAME

Despite the archive.org title listing "Robin Hood", this disc contains
**Robin Hood: Defender of the Crown** (volume ID: RHDOTC2), not Legend of Sherwood.

    source:   https://archive.org/details/score152dvd

## Windows — GOG/romsfun repackage (v2.0.0.12)

GOG-style installer with bonus content (artworks, avatars, manual, soundtrack, wallpapers).

    filename: robin-hood-romsfun.zip (contains setup_robin_hood_2.0.0.12.exe, 577 MB)
    format:   Zip archive (641 MB total)
    source:   https://romsfun.com/download/robin-hood-the-legend-of-sherwood-196025
    MD5:      d755e50c1a1a5c292c277f2808c92c2f
    SHA256:   479673d404d3bf697417b9809ce10dc7bf2ad8ef054982adae3eded1fb250de1

## Windows — GOG v1.1 hotfix (gogunlocked repackage)

GOG installer with v1.1 hotfix (build 24778). InnoSetup exe + bin. Same extras as romsfun.

    filename: robinhood-gogunlocked.zip (contains setup_robin_hood_-_the_legend_of_sherwood_1.1_hotfix_(24778).exe + .bin, 636 MB)
    format:   Zip archive (701 MB total)
    source:   https://gogunlocked.com/robin-hood-the-legend-of-sherwood-free-download/
    MD5:      00292c14650583a9b350ce23decda5b4
    SHA256:   7451b367a9713f43fad9a3f2b9c6a1d9af382a9a97809f676bd0aa9cd9a0c028

## ZETA OS — BeOS/ZETA port

Port for ZETA OS (a BeOS derivative). BFS filesystem disc image containing `RobinHood.zpkg`,
`Manual.pdf`, `Handbuch.pdf` (German manual), and `ReadME.txt`.

    filename: Robin Hood The Legend of Sherwood ZETA.iso
    format:   BFS disc image (557 MB)
    source:   https://archive.org/download/robin-hood-the-legend-of-sherwood-zeta
    MD5:      7e4fbec45cd6d42c69bab32fbabd6903
    SHA256:   e96c332b61178ea4d03423c60ddc0086e4bc4589f316c4e3240625a1443b187d

## Redump — full release disc images

| Redump | Title | Region | Lang | Version | Edition | Serial | Mode | Size | CRC-32 | MD5 | SHA-1 | Mastering Code | Mastering SID | Mould SID |
|--------|-------|--------|------|---------|---------|--------|------|------|--------|-----|-------|----------------|---------------|-----------|
| [40703](http://redump.org/disc/40703/) | Robin Hood: The Legend of Sherwood | USA | English | 1.0 | Original | 24423CD | Mode 2 | 747,557,328 | `7d4e4cae` | `5d0e420693f5ed48c89246f05ad65d94` | `77ddff6fbe515388c04874a0a978f9cc9b5dcfb1` | 1AYM5\<9265\>24423 | IFPI L485 | IFPI 81C1, IFPI 818C |
| [35111](http://redump.org/disc/35111/) | Robin Hood: Legenda Sherwood | Poland | Polish | 1.1 | Nowa eXtra Klasyka | — | Mode 1 | 824,213,712 | `d66312b9` | `3305bb13fbcad234f0b17326b8de3b32` | `157830fe7b7ea9f00aba8ab1ef6229e3718c3e0c` | CDPROJEKT 1900697503 RBINHB Robin Hood GM Records | IFPI LT57 | IFPI Z901 |
| [113363](http://redump.org/disc/113363/) | Robin Hood: Die Legende von Sherwood | Germany | German | 1.0 | Back to Games | CD 60812 | Mode 1 | 812,286,720 | `528e254b` | `80950711ca5b4e995b750e3992493b66` | `e3b8ce9b2786051d50bb80c6c4d64ba76e018a8f` | MPO CR 60812ROBINALPHA @ 11/19/06 | IFPI L039 | IFPI 1263 |

# Linux Full Release — Runesoft

The license for releasing a Linux version of this game was at some point sold to Runesoft — a German company specializing in porting Windows games to Linux.

They released at least three different versions:

## Linux version v1.0

## Linux version v1.1

## Linux version v1.2

Multilingual, CD-ROM edition. 32-bit x86 ELF executable (MojoSetup installer).

    filename: robin.hood_1.2-multilingual_x86-20121114.mojo.run
    format:   MojoSetup ELF installer (832 MB)
    MD5:      a691372b2894d85ad66f22ceb750795f
    SHA256:   a9beda295cd1a34b4ede838caefdd0db93729aee3139bf9e4e42562c56107f62

Known bugs (from Runesoft's issue tracker): French/German language selection is swapped,
cutscene dialogues fail with "File!Not!Found" errors.

## Unsorted Runesoft notes

> https://bitbucket.org/runesoftdev/robinhood_public/issues

> Using robin.hood_1.2-multilingual.cdrom_x86-20121114.mojo.run the game is installed in the wrong language when selecting either French or German as the version on CD (English not tested), i.e. when selecting French the ingame texts are in German and vice versa. I should probably mention that the cutscene dialogues also aren't working any more - whenever a cutscene should play an error message similar to "File!Not!Found Data/Text/Dialogues/DLG_..." (written from my memory) appears.

> When using robin.hood_1.2-multilingual.cdrom_x86-20121108.mojo.run to create a system wide installation of the German version of the game, the following files are created with wrong permissions: * 1031/Data/Interface/Slideshow_in.pak * 1031/Data/Text/Level.res

> SDL_VIDEO_X11_XRANDR is enabled by default now. Get the 20121105 update, either from Desura or if you own the Linux retail cdrom from https://bitbucket.org/runesoftdev/robinhood_public/downloads
> https://bitbucket.org/runesoftdev/robinhood_public/issues/1/no-sound-and-game-hangs-on-quit

> The game is only designed to work with 640x480, 800x600 and 1024x768. In the screen-shot you can see one of the many drawbacks of supporting other and even wide screen resolutions. The whole GUI only works for these fixed resolutions and it is not easy to change.

> Sorry, this looks like a Unity/Ubuntu specific problem and we do not know how to fix it with SDL 1.2. We will have a look at this as soon as we migrate Robin Hood to SDL 2.0.

> Multi language support will be available with the next update. Currently we are porting Robin Hood from SDL 1.2 to SDL 2.0. We are planning to release an update after that. (2013-05-30)

(i don't think this ever happened?)
