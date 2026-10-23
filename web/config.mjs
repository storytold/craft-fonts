// Set window.CraftFontsConfig before importing any library modules.
const supplied = window.CraftFontsConfig || {};
export const baseURL = new URL(supplied.baseURL || './', import.meta.url);
export const assetBaseURL = new URL(supplied.assetBaseURL || './', baseURL);
export const catalogURL = new URL(supplied.catalogURL || 'catalog.json', baseURL);
export const discoveryURL = new URL(supplied.discoveryURL || 'fontlibrary-catalog.json', baseURL);
export const databaseName = supplied.databaseName || 'craft-font-library';
export const autoActivate = supplied.autoActivate === true;
export function assetURL(path) {
  const url = new URL(path, assetBaseURL);
  if (url.origin !== location.origin || !url.pathname.startsWith(assetBaseURL.pathname) || url.search || url.hash) {
    throw Error('Catalog assets must stay inside the configured same-origin asset directory.');
  }
  return url;
}
export const supportURL = supplied.supportURL || 'https://github.com/storytold/craft-fonts/blob/main/docs/weblauncher.md';
