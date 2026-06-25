import type { TemplateValueConfig } from "../config/claudeProviderPresets";

const isPlainObject = (value: unknown): value is Record<string, any> => {
  return Object.prototype.toString.call(value) === "[object Object]";
};

const deepMerge = (
  target: Record<string, any>,
  source: Record<string, any>,
): Record<string, any> => {
  Object.entries(source).forEach(([key, value]) => {
    if (isPlainObject(value)) {
      if (!isPlainObject(target[key])) {
        target[key] = {};
      }
      deepMerge(target[key], value);
    } else {
      target[key] = value;
    }
  });
  return target;
};

const deepRemove = (
  target: Record<string, any>,
  source: Record<string, any>,
) => {
  Object.entries(source).forEach(([key, value]) => {
    if (!(key in target)) return;

    if (isPlainObject(value) && isPlainObject(target[key])) {
      deepRemove(target[key], value);
      if (Object.keys(target[key]).length === 0) {
        delete target[key];
      }
    } else if (isSubset(target[key], value)) {
      delete target[key];
    }
  });
};

const isSubset = (target: any, source: any): boolean => {
  if (isPlainObject(source)) {
    if (!isPlainObject(target)) return false;
    return Object.entries(source).every(([key, value]) =>
      isSubset(target[key], value),
    );
  }

  if (Array.isArray(source)) {
    if (!Array.isArray(target) || target.length !== source.length) return false;
    return source.every((item, index) => isSubset(target[index], item));
  }

  return target === source;
};

const deepClone = <T>(obj: T): T => {
  if (obj === null || typeof obj !== "object") return obj;
  if (obj instanceof Date) return new Date(obj.getTime()) as T;
  if (obj instanceof Array) return obj.map((item) => deepClone(item)) as T;
  if (obj instanceof Object) {
    const clonedObj = {} as T;
    for (const key in obj) {
      if (obj.hasOwnProperty(key)) {
        clonedObj[key] = deepClone(obj[key]);
      }
    }
    return clonedObj;
  }
  return obj;
};

export interface UpdateCommonConfigResult {
  updatedConfig: string;
  error?: string;
}

export const validateJsonConfig = (
  value: string,
  fieldName: string = "配置",
): string => {
  if (!value.trim()) {
    return "";
  }
  try {
    const parsed = JSON.parse(value);
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) {
      return `${fieldName}必须是 JSON 对象`;
    }
    return "";
  } catch {
    return `${fieldName}JSON格式错误，请检查语法`;
  }
};

export const updateCommonConfigSnippet = (
  jsonString: string,
  snippetString: string,
  enabled: boolean,
): UpdateCommonConfigResult => {
  let config: Record<string, any>;
  try {
    config = jsonString ? JSON.parse(jsonString) : {};
  } catch {
    return {
      updatedConfig: jsonString,
      error: "配置 JSON 解析失败，无法写入通用配置",
    };
  }

  if (!snippetString.trim()) {
    return {
      updatedConfig: JSON.stringify(config, null, 2),
    };
  }

  const snippetError = validateJsonConfig(snippetString, "通用配置片段");
  if (snippetError) {
    return {
      updatedConfig: JSON.stringify(config, null, 2),
      error: snippetError,
    };
  }

  const snippet = JSON.parse(snippetString) as Record<string, any>;

  if (enabled) {
    const merged = deepMerge(deepClone(config), snippet);
    return {
      updatedConfig: JSON.stringify(merged, null, 2),
    };
  }

  const cloned = deepClone(config);
  deepRemove(cloned, snippet);
  return {
    updatedConfig: JSON.stringify(cloned, null, 2),
  };
};

export const hasCommonConfigSnippet = (
  jsonString: string,
  snippetString: string,
): boolean => {
  try {
    if (!snippetString.trim()) return false;
    const config = jsonString ? JSON.parse(jsonString) : {};
    const snippet = JSON.parse(snippetString);
    if (!isPlainObject(snippet)) return false;
    return isSubset(config, snippet);
  } catch {
    return false;
  }
};

export const getApiKeyFromConfig = (jsonString: string): string => {
  try {
    const config = JSON.parse(jsonString);

    if (
      typeof config?.apiKey === "string" &&
      config.apiKey &&
      !config.apiKey.includes("${")
    ) {
      return config.apiKey;
    }

    const env = config?.env;
    if (!env) return "";

    const token = env.ANTHROPIC_AUTH_TOKEN;
    const apiKey = env.ANTHROPIC_API_KEY;
    return typeof token === "string"
      ? token
      : typeof apiKey === "string"
        ? apiKey
        : "";
  } catch {
    return "";
  }
};

export const applyTemplateValues = (
  config: any,
  templateValues: Record<string, TemplateValueConfig> | undefined,
): any => {
  const resolvedValues = Object.fromEntries(
    Object.entries(templateValues ?? {}).map(([key, value]) => {
      const resolvedValue =
        value.editorValue !== undefined
          ? value.editorValue
          : (value.defaultValue ?? "");
      return [key, resolvedValue];
    }),
  );

  const replaceInString = (str: string): string => {
    return Object.entries(resolvedValues).reduce((acc, [key, value]) => {
      const placeholder = `\${${key}}`;
      if (!acc.includes(placeholder)) {
        return acc;
      }
      return acc.split(placeholder).join(value ?? "");
    }, str);
  };

  const traverse = (obj: any): any => {
    if (typeof obj === "string") {
      return replaceInString(obj);
    }
    if (Array.isArray(obj)) {
      return obj.map(traverse);
    }
    if (obj && typeof obj === "object") {
      const result: any = {};
      for (const [key, value] of Object.entries(obj)) {
        result[key] = traverse(value);
      }
      return result;
    }
    return obj;
  };

  return traverse(config);
};

export const hasApiKeyField = (jsonString: string): boolean => {
  try {
    const config = JSON.parse(jsonString);

    if (Object.prototype.hasOwnProperty.call(config, "apiKey")) {
      return true;
    }

    const env = config?.env ?? {};
    return (
      Object.prototype.hasOwnProperty.call(env, "ANTHROPIC_AUTH_TOKEN") ||
      Object.prototype.hasOwnProperty.call(env, "ANTHROPIC_API_KEY")
    );
  } catch {
    return false;
  }
};

export const setApiKeyInConfig = (
  jsonString: string,
  apiKey: string,
  options: {
    createIfMissing?: boolean;
    apiKeyField?: string;
  } = {},
): string => {
  const { createIfMissing = false, apiKeyField } = options;
  try {
    const config = JSON.parse(jsonString);

    if (Object.prototype.hasOwnProperty.call(config, "apiKey")) {
      config.apiKey = apiKey;
      return JSON.stringify(config, null, 2);
    }

    if (!config.env) {
      if (!createIfMissing) return jsonString;
      config.env = {};
    }
    const env = config.env as Record<string, any>;

    if ("ANTHROPIC_AUTH_TOKEN" in env) {
      env.ANTHROPIC_AUTH_TOKEN = apiKey;
    } else if ("ANTHROPIC_API_KEY" in env) {
      env.ANTHROPIC_API_KEY = apiKey;
    } else if (createIfMissing) {
      env[apiKeyField ?? "ANTHROPIC_AUTH_TOKEN"] = apiKey;
    } else {
      return jsonString;
    }
    return JSON.stringify(config, null, 2);
  } catch {
    return jsonString;
  }
};
