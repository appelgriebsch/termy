import { serve } from "@hono/node-server";
import { serveStatic } from "@hono/node-server/serve-static";
import { Notra } from "@usenotra/sdk";
import { Hono, type Context } from "hono";

const app = new Hono();

const NOTRA_ORG_ID = process.env.NOTRA_ORG_ID;
const NOTRA_API_KEY = process.env.NOTRA_API_KEY;
const PORT = Number(process.env.PORT) || 3000;

const notra = NOTRA_API_KEY
  ? new Notra({ bearerAuth: NOTRA_API_KEY })
  : null;

interface NotraConfig {
  client: Notra;
  organizationId: string;
}

function respondNotConfigured(context: Context): Response {
  return context.json({ error: "Notra not configured" }, 500);
}

function getNotraConfig(context: Context): NotraConfig | Response {
  if (!notra || !NOTRA_ORG_ID) {
    return respondNotConfigured(context);
  }

  return {
    client: notra,
    organizationId: NOTRA_ORG_ID,
  };
}

app.get("/api/changelogs", async (c) => {
  const config = getNotraConfig(c);
  if (config instanceof Response) {
    return config;
  }

  try {
    const result = await config.client.content.listPosts({
      organizationId: config.organizationId,
      status: "published",
      contentType: "changelog",
      sort: "desc",
      limit: 100,
    });

    c.header("Cache-Control", "public, max-age=300");
    return c.json(result);
  } catch {
    return c.json({ error: "Failed to fetch changelogs from Notra" }, 502);
  }
});

app.get("/api/changelogs/:id", async (c) => {
  const config = getNotraConfig(c);
  if (config instanceof Response) {
    return config;
  }

  try {
    const result = await config.client.content.getPost({
      organizationId: config.organizationId,
      postId: c.req.param("id"),
    });

    c.header("Cache-Control", "public, max-age=300");
    return c.json(result);
  } catch {
    return c.json({ error: "Failed to fetch changelog from Notra" }, 502);
  }
});

app.use("/*", serveStatic({ root: "./dist" }));

// SPA fallback - serve index.html for client-side routes
app.get("/*", serveStatic({ root: "./dist", path: "index.html" }));

serve({ fetch: app.fetch, port: PORT }, () => {
  console.log(`Server running on port ${PORT}`);
});
