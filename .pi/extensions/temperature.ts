import type { ExtensionAPI } from "@mariozechner/pi-coding-agent";

export default function (pi: ExtensionAPI) {
  let temperature: number | null = null;

  // ── Helpers ───────────────────────────────────────────────────────────
  function updateStatus(ctx: {
    ui: { setStatus: (key: string, text: string | undefined) => void };
  }) {
    ctx.ui.setStatus("temperature", temperature !== null ? `temp: ${temperature}` : undefined);
  }

  function showResetWidget(
    ctx: {
      ui: { setWidget: (key: string, lines: string[] | undefined) => void };
    },
    message: string,
  ) {
    ctx.ui.setWidget("temperature", [message]);
    // Auto-clear after a few seconds (the widget stays until the next
    // user input, but this ensures it doesn't linger forever)
    setTimeout(() => {
      // Only clear if we're showing the reset message (don't clear
      // if user set a new temperature in the meantime)
      ctx.ui.setWidget("temperature", undefined);
    }, 4000);
  }

  // ── Reset on session changes ──────────────────────────────────────────
  // Covers: startup, /new, /resume, /fork, /reload
  pi.on("session_start", async (_event, ctx) => {
    if (temperature !== null) {
      temperature = null;
      updateStatus(ctx);
      showResetWidget(ctx, "Temperature reset to model default");
    }
  });

  // ── Reset on model switch ─────────────────────────────────────────────
  pi.on("model_select", async (_event, ctx) => {
    if (temperature !== null) {
      temperature = null;
      updateStatus(ctx);
      showResetWidget(ctx, "Temperature reset to model default");
    }
  });

  // ── Inject temperature into every provider request ────────────────────
  pi.on("before_provider_request", (event, _ctx) => {
    if (temperature !== null) {
      return { ...event.payload, temperature };
    }
  });

  // ── /temperature command ──────────────────────────────────────────────
  pi.registerCommand("temperature", {
    description:
      "Set the temperature (entropy) for all models (0–2). " +
      "Run without arguments to show the current value. " +
      "Use 'reset' to revert to model default.",
    handler: async (args, ctx) => {
      const trimmed = args?.trim() ?? "";

      // ── No arguments: show current value ──────────────────────────
      if (!trimmed) {
        if (temperature === null) {
          ctx.ui.notify("Temperature: not set (using model default)", "info");
        } else {
          ctx.ui.notify(`Temperature: ${temperature}`, "info");
        }
        return;
      }

      // ── Reset ────────────────────────────────────────────────────
      if (trimmed.toLowerCase() === "reset") {
        if (temperature !== null) {
          temperature = null;
          ctx.ui.notify("Temperature reset to model default", "info");
        } else {
          ctx.ui.notify("Temperature was not set (already using model default)", "info");
        }
        updateStatus(ctx);
        return;
      }

      // ── Set new value ────────────────────────────────────────────
      const value = parseFloat(trimmed);
      if (isNaN(value) || !isFinite(value)) {
        ctx.ui.notify(
          `Invalid temperature: "${trimmed}". Must be a number between 0 and 2.`,
          "error",
        );
        return;
      }
      if (value < 0 || value > 2) {
        ctx.ui.notify(
          `Temperature ${value} is out of range. Must be between 0 and 2.`,
          "error",
        );
        return;
      }

      temperature = value;
      updateStatus(ctx);
      ctx.ui.notify(`Temperature set to ${temperature}`, "success");
    },
  });
}
