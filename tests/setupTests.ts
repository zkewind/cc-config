import "@testing-library/jest-dom";
import { afterAll, afterEach, beforeAll, vi } from "vitest";
import { cleanup } from "@testing-library/react";
import i18n from "i18next";
import { initReactI18next } from "react-i18next";
import { server } from "./msw/server";
import { resetProviderState } from "./msw/state";
import { resetTauriEventListeners } from "./msw/tauriMocks";
import "./msw/tauriMocks";

beforeAll(async () => {
  server.listen({ onUnhandledRequest: "warn" });
  await i18n.use(initReactI18next).init({
    lng: "zh",
    fallbackLng: "zh",
    resources: {
      zh: { translation: {} },
      en: { translation: {} },
    },
    interpolation: {
      escapeValue: false,
    },
  });
});

afterEach(async () => {
  cleanup();
  resetTauriEventListeners();
  resetProviderState();
  server.resetHandlers();
  localStorage.clear();
  await i18n.changeLanguage("zh");
  vi.clearAllMocks();
});

afterAll(() => {
  server.close();
});
