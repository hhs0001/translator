import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

import en from './locales/en.json';
import ptBR from './locales/pt-BR.json';

export const resources = {
  en: { translation: en },
  'pt-BR': { translation: ptBR },
} as const;

export type Language = keyof typeof resources;

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources,
    fallbackLng: 'en',
    defaultNS: 'translation',
    interpolation: {
      escapeValue: false, // React already escapes values
    },
    detection: {
      order: ['localStorage', 'navigator'],
      caches: ['localStorage'],
      lookupLocalStorage: 'language',
    },
  });

export default i18n;

// Helper to change language and persist
export const changeLanguage = async (lng: Language) => {
  await i18n.changeLanguage(lng);
  localStorage.setItem('language', lng);
};
