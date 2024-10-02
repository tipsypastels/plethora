declare global {
  var Stimulus: import("@hotwired/stimulus").Application;
}

import { Application } from "@hotwired/stimulus";
import { ToastsController } from "../controllers/toasts";

window.Stimulus ??= Application.start();

Stimulus.register("toasts", ToastsController);
