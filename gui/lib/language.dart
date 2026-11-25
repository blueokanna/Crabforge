import 'package:flutter/material.dart';

import 'l10n/app_localizations.dart';

extension ConsoleStrings on BuildContext {
  AppLocalizations get strings => AppLocalizations.of(this);
}

// Keep the picker in English alphabetical order and show each native name.
const languageNames = {
  'zh': '中文',
  'en': 'English',
  'fr': 'Français',
  'de': 'Deutsch',
  'hi': 'हिन्दी',
  'ja': '日本語',
  'ko': '한국어',
  'pt': 'Português',
  'es': 'Español',
};
