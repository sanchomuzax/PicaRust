# PicaRust funkcióleltár: különbségek és kézi átnézés

A `funkcioleltar.csv` mellékterméke (agent #377). Gépi, ismételhető; kézzel ne szerkeszd.
Elemzett fájlok: 351 QML, 2564 tesztfájl.

## 1. Csak a QML-ben (egyetlen tesztfájl sem hivatkozik rá)

19 sor, területenként (a sorok a CSV-ben `forras_teszt` üresen):

- RecentlyUpdatedContextMenu: 7
- EditorDialogs: 3
- UpdatePanel: 2
- CollagePanelTabButton: 1
- CollageSnapColumn: 1
- CollageZOrderColumn: 1
- ColorSpectrumPicker: 1
- DateCalendarPopup: 1
- MovieTabButton: 1
- TrayActionButton: 1

Szigorúbb mérték: 96 sornak van `objectName`-je, de egyetlen teszt sem nevezi meg (itt a fájl-szintű hivatkozás nem számít). Legnagyobb 25 terület:

- CaptureMoviePanelPopup: 15
- PhotoViewerTopBar: 9
- MovieTitleDialog: 8
- ImportSourcePreview: 6
- FtpAccountDialog: 5
- FolderContextMenu: 4
- RecentlyUpdatedContextMenu: 4
- UnnamedFacesView: 4
- ImportSourceDialog: 3
- OptionsTabGooglePhotos: 3
- PicasaToolsMenu: 3
- VideoEditPanel: 3
- WebExportDialog: 3
- DedupDialog: 2
- EffectSliceBox: 2
- FolderManagerDialog: 2
- MovieMakerOptions: 2
- MovieMakerPreview: 2
- UpdatePanel: 2
- CollageDialogs: 1
- ColorSpectrumPicker: 1
- EditorParamPanel: 1
- EmailChoiceDialog: 1
- FolderHierarchyView: 1
- MainWindowCommands: 1

## 2. Csak a tesztben

### 2.1 Nem létező QML-fájlra hivatkozó tesztek (48 név)

- `A.qml`: tests/app/test_qml_eloforditas_hatarido_6374.py, tests/tools/test_check_silent_qml_controls_4438.py
- `AsyncThumbDelegate.qml`: tests/app/qml_functional/test_thumbnail_scroll_responsive_7086.py
- `B.qml`: tests/app/test_qml_eloforditas_hatarido_6374.py
- `BackupDialog.qml`: tests/app/test_mentes_terv_hatterszal_3645.py
- `BackupFolderStrip.qml`: tests/app/qml_functional/test_mentes_konyvtar_szuro_3681.py
- `Bar.qml`: tests/tools/test_kepesseg_or_1476.py
- `Button.qml`: tests/app/qml_functional/test_warning_guard.py, tests/support/qml_warning_filter.py, tests/test_csomag_meres_grafika_6426.py
- `CheckBox.qml`: tests/app/qml_functional/test_warning_guard.py
- `ContextMenu.qml`: tests/app/qml_functional/test_menu_invariansok_886.py
- `Dialog.qml`: tests/app/qml_functional/test_kotesi_hurok_1185.py, tests/tools/test_check_dead_signals_1003.py
- `Dlg.qml`: tests/test_profil_valtozas_6374.py
- `Elso.qml`: tests/tools/test_kepesseg_or_1476.py
- `Fo.qml`: tests/tools/test_check_dead_signals_1003.py
- `Idegen.qml`: tests/tools/test_kepesseg_or_1476.py
- `MainSplit.qml`: tests/test_main_split_component_5955.py
- `Masik.qml`: tests/tools/test_kepesseg_or_1476.py
- `Masodik.qml`: tests/tools/test_kepesseg_or_1476.py
- `Menu.qml`: tests/tools/test_kepesseg_or_1476.py
- `Mezo.qml`: tests/tools/test_kepesseg_or_1476.py
- `Nezet.qml`: tests/tools/test_kepesseg_or_1476.py
- `One.qml`: tests/tools/test_check_dead_signals_1003.py
- `Other.qml`: tests/tools/test_check_silent_qml_controls_4438.py
- `Panel.qml`: tests/tools/test_kepesseg_or_1476.py
- `PeoplePane.qml`: tests/test_csomag_meres_grafika_6426.py
- `PhotoArea.qml`: tests/tools/test_check_silent_qml_controls_4438.py
- `PressFeedbackMouseArea.qml`: tests/app/qml_functional/test_gombleltar_es_lenyomas_6381.py
- `PressFeedbackTapHandler.qml`: tests/app/qml_functional/test_gombleltar_es_lenyomas_6381.py
- `Proba.qml`: tests/tools/test_qml_undefined_or_1572.py
- `Sajat.qml`: tests/app/qml_functional/test_warning_guard.py
- `TesztHelyterkep.qml`: tests/app/qml_functional/test_places_loading_geometry_5275.py, tests/app/qml_functional/test_places_loading_offline_4584.py, tests/app/qml_functional/test_places_marker_showphotos_4575.py
- `Two.qml`: tests/tools/test_check_dead_signals_1003.py
- `UjParbeszed1490.qml`: tests/tools/test_kepesseg_or_1476.py
- `UresUj.qml`: tests/tools/test_kepesseg_or_1476.py
- `ViewerLayout.qml`: tests/tools/test_check_silent_qml_controls_4438.py
- `Widget.qml`: tests/tools/test_check_silent_qml_controls_4438.py
- `a.qml`: tests/scripts/test_nyelvi_ellenorzes.py
- `borito.qml`: tests/app/test_mappa_borito_ekezetes_ut_3352.py
- `capture.qml`: tests/app/qml_functional/test_snapshot_capture_6332.py
- `fulikon_4846.qml`: tests/app/qml_functional/test_fulikon_rajz_4846.py
- `fulsav_4552.qml`: tests/app/qml_functional/test_fulsav_ikonmeret_4552.py
- ... és még 8 (a teljes lista a CSV-ből szűrhető)

### 2.2 Keresőhívásban használt, a QML-ben sehol nem szereplő név (55 név)

Heurisztika: a névnek nincs egyezése a QML-szövegben, és nem prefixe sem; a tesztben maga deklarált nevek kiestek. A dinamikusan képzett nevek hamis jelzést adhatnak.

- `addNameButton`: tests/app/qml_functional/test_unnamed_faces_view_5183.py
- `adoptPicasaFoldersButton`: tests/app/test_qml_picasa_import.py
- `albumsHeaderRow`: tests/app/qml_functional/test_folder_collections.py, tests/app/qml_functional/test_folder_pane_scroll_730_731_732.py
- `contextMenuFindSimilar`: tests/app/qml_functional/test_kereses_menu_3468.py
- `contextMenuRename`: tests/app/test_qml_context_menu.py
- `cropAspectDelete13`: tests/app/qml_functional/test_egyeni_arany_kuka_4550.py
- `cropSuggestion0Thumb`: tests/app/qml_functional/test_editor_crop_aspect_448.py
- `customCollection_Munka`: tests/app/qml_functional/test_folder_pane_custom_collections_476.py
- `customCollection_Nyaralások`: tests/app/qml_functional/test_folder_pane_custom_collections_476.py
- `customCollection_NyaralásokRow`: tests/app/qml_functional/test_folder_pane_custom_collections_476.py
- `editTabLegacy`: tests/app/qml_functional/test_editor_legacy_tab_571.py, tests/app/qml_functional/test_orokolt_ful_betumetrika_3263.py
- `effectBwThumb`: tests/app/qml_functional/test_editor_look.py
- `effectNightVisionThumb`: tests/app/qml_functional/test_effektracs_hisztogram_6477.py
- `effectSepiaThumb`: tests/app/qml_functional/test_editor_look.py, tests/app/qml_functional/test_effect_tile_grid_704.py
- `effectsUndoButton`: tests/app/qml_functional/test_editor_look.py
- `expandGroupsCheck`: tests/app/qml_functional/test_unnamed_faces_view.py
- `faceNameCancel`: tests/app/qml_functional/test_viewer_unnamed_face_4572.py
- `faceNameOk`: tests/app/qml_functional/test_viewer_unnamed_face_4572.py
- `fillLightComingSoon`: tests/app/test_qml_editor_panel.py
- `fillLightRow`: tests/app/test_qml_editor_panel.py
- `folderContextMenuSetDate`: tests/app/test_folder_context_menu_320.py
- `folderPaneContextMenuHandler`: tests/app/qml_functional/test_hasab_ures_jobbklikk_7349.py, tests/app/qml_functional/test_racs_maradek_valodi_5150_p.py
- `folderPaneHeaderRow`: tests/app/qml_functional/test_folder_collections.py
- `groupByFaceCheck`: tests/app/qml_functional/test_unnamed_faces_view.py
- `ignoredFaceImage_1`: tests/app/qml_functional/test_nev_nelkuliek_elrendezes_5324.py
- `ignoredFaceNameField_1`: tests/app/qml_functional/test_nev_nelkuliek_elrendezes_5324.py
- `legacyEffectsIntro`: tests/app/qml_functional/test_editor_legacy_tab_571.py, tests/app/qml_functional/test_orokolt_bevezeto_3278.py
- `menuFileMoveToCollection`: tests/app/qml_functional/test_kereses_es_gyujtemeny_folyamatok_6845.py
- `menuFileUndoSave`: tests/app/qml_functional/test_save_dialogs_444.py
- `otherHeaderRow`: tests/app/qml_functional/test_folder_pane_scroll_730_731_732.py
- `peopleAlbumMenuSortAlbumBy`: tests/app/qml_functional/test_people_album_menu_4645.py
- `peopleHeaderRow`: tests/app/qml_functional/test_folder_collections.py
- `peopleManagerContact_Bela`: tests/app/qml_functional/test_people_manager_5182.py
- `peopleManagerContact_Nagykiss`: tests/app/qml_functional/test_people_manager_5437.py
- `personThumbnailImage_Anna`: tests/app/qml_functional/test_person_album_thumbnail_4536.py
- `photoContextMenuLoader`: tests/app/qml_functional/test_halasztott_nezetek_6700.py
- `picasaImportDrivePrefixField`: tests/app/qml_functional/test_picasa_import_windows_qml_6656.py
- `picasaImportLocalPrefixField`: tests/app/qml_functional/test_picasa_import_windows_qml_6656.py
- `printContactSheetRadio`: tests/app/qml_functional/test_nyomtatas_egyseges_meretlista_3712.py
- `printOnePerPageRadio`: tests/app/qml_functional/test_nyomtatas_egyseges_meretlista_3712.py
- ... és még 15 (a teljes lista a CSV-ből szűrhető)

## 3. Meglévő `docs/funkcio-leltar.md` és a QML

Témakör-szintű összevetés (a `spec_tabla.py` mintái szerint), nem sorról sorra.

### 3.1 A meglévő leltárban van, a QML-ben nincs hozzá témakör (4 sor)

- Keresősáv, szűrők (★ ☺ ▤ 📍) (Keresés, szűrés, albumok)
- color: szín-keresés + avgcolor (Keresés, szűrés, albumok)
- „Keresés mentése albumként", „Címke albumként" (Keresés, szűrés, albumok)
- Hasonlósági keresés (HOG-alapú, előre épített index) (Negyedik-ötödik kör - a menürendszer teljes átfésülése (2026-08-07))

### 3.2 A meglévő leltár sora témakörhöz nem rendelhető (88 sor)

Ezek jellemzően adatformátum- és kutatási sorok, nincs közvetlen QML-megfelelőjük.

### 3.3 QML-témakör, amelyhez nincs sor a meglévő leltárban (1 téma)

- picasa-elso-inditas.md: 2 QML-sor

### 3.4 Témakör nélküli QML-fájlok (86 db)

AaUtkozesDialog, AboutDialog, AddCustomAspectRatioDialog, AddFileDialog, AdjustTimestampDialog, AlbumContextMenu, BatchEditProgressPanel, CollectionContextMenu, CollectionPasswordDialog, ColorSpectrumPicker, CompactDatabaseDialog, ConfigureButtonsDialog, ConfirmDialog, CreateResultDialog, DateCalendarPopup, DedupDialog, DocumentTabStrip, EditOverwriteDialog, EffectSliceBox, EndEditModalityDialog, FileOpsDialogs, FtpAccountDialog, GiftCdHost, HelpDialog, HiddenPasswordDialog, ImmediateDeleteDialog, InitialScanDialog, KeywordsDialog, LightboxHeaderButtons, LightboxHeaderSuggestionRow, LightboxHeaderTitle, MainDialogs, MainErrorBanner, MainToolbar, MainWindowCommands, MainWindowDialogStack, MenuItem, MoveDatabaseDialog, NameInputDialog, NewCollectionDialog, OpenWithDialog, OptionsDialog, OptionsTabFileTypes, OptionsTabGeneral, OptionsTabGooglePhotos, OptionsTabNameTags, OptionsTabNetwork, PassportErrorDialog, PerfMonitorPanel, PhotoContextMenu, PhotoViewerContextActions, PhotoViewerEditWorkflow, PhotoViewerOverlayControls, PhotoViewerQuickToolbar, PhotoViewerSettingsDialog, PhotoViewerTopBar, PicasaButton, PicasaComboBox, PicasaCreateMenu, PicasaEditMenu, PicasaFileMenu, PicasaHelpMenu, PicasaMenu, PicasaMenuBar, PicasaMenuItem, PicasaPictureMenu, PicasaToolsMenu, PicasaViewMenu, PosterCreateDialog, PropertiesPanel, PublishPanel, QuickTagsConfigDialog, RecentlyUpdatedContextMenu, RightDrawer, SaveDialogs, ScreensaverDialog, TagContextMenu, TagsPanel, TesztuzemNaploDialog, TextFieldContextMenu, UpdatePanel, VideoEditPanel, VideoPlayerControls, VideoPlayerView, ViewerContextMenu, WindowHelp

## 4. Kézi átnézést kérő sorok (besorolatlan, 6-os mérföldkő)

24 sor: egyetlen besorolási minta sem illett rájuk, a 6-ost kapták.

- `button:DateCalendarPopup:datePickerNextMonth`: ›
- `button:DateCalendarPopup:datePickerPreviousMonth`: ‹
- `button:PicasaButton:control`: control
- `combobox:PicasaComboBox:control`: control
- `dialog:AaUtkozesDialog:aaUtkozesDialog`: Choose Edits
- `dialog:CreateResultDialog:createResultDialog`: Create
- `dialog:NameInputDialog:nameInputDialog`: nameInputDialog
- `filedialog:AddFileDialog:addFileDialog`: Add File to PicasaPy...
- `leltar:13:a-szerkeszto-ot-fulenek-gombsorrendje`: A szerkesztő öt fülének gombsorrendje
- `leltar:13:canon-live-view-panel`: Canon live-view panel
- `leltar:13:hattermuvelet-jelzo-megszakitassal`: Háttérművelet-jelző megszakítással
- `leltar:13:picasa-notifier-recent-activity`: „Picasa Notifier" / Recent Activity
- `leltar:13:redo-ini-kulcs-szemantikaja`: redo= ini-kulcs szemantikája
- `leltar:13:visszavonas-modell-harom-kulon-verem`: Visszavonás-modell (HÁROM külön verem)
- `leltar:13:webkamera-felvetel-kepernyokep-rogzites`: Webkamera-felvétel, képernyőkép-rögzítés
- `menu:PicasaMenu:menu`: menu
- `menuitem:MenuItem:control`: control
- `menuitem:PicasaMenuItem:control`: control
- `picasabutton:AaUtkozesDialog:aaUtkozesElsoButton`: aaUtkozesElsoButton
- `picasabutton:AaUtkozesDialog:aaUtkozesMasodikButton`: aaUtkozesMasodikButton
- `picasabutton:AaUtkozesDialog:aaUtkozesMegseButton`: Cancel
- `popup:ColorSpectrumPicker:colorSpectrumPicker`: colorSpectrumPicker
- `popup:DateCalendarPopup:root`: root
- `popup:PicasaComboBox:picasaComboPopup`: picasaComboPopup

## Darabszámok

Összesen: 1608 sor.

### Típus szerint

- menu: 387
- gomb: 520
- parbeszed: 165
- billentyu: 130
- effekt: 84
- beallitas: 184
- egyeb: 138

### Mérföldkő szerint

- 2 Böngészés: 528
- 3 Képnéző és arcok: 210
- 4 Szerkesztő: 273
- 5 Kimenetek: 334
- 6 Kör bezárása: 263

### Típus x mérföldkő

| típus | 2 | 3 | 4 | 5 | 6 |
|---|---|---|---|---|---|
| menu | 206 | 51 | 22 | 35 | 73 |
| gomb | 132 | 97 | 111 | 128 | 52 |
| parbeszed | 59 | 15 | 17 | 40 | 34 |
| billentyu | 42 | 17 | 4 | 10 | 57 |
| effekt | 0 | 0 | 84 | 0 | 0 |
| beallitas | 38 | 15 | 8 | 94 | 29 |
| egyeb | 51 | 15 | 27 | 27 | 18 |
