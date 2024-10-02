import { Controller } from "@hotwired/stimulus";

const DEFAULT_TIMEOUT = 4000; // ms

export type ToastType = "info" | "warning" | "error";

export interface Toast {
  type: ToastType;
  msg: string;
  timeout?: number | false;
}

let instance: ToastsController | undefined;

export class ToastsController extends Controller<HTMLElement> {
  static classes = ["base", "info", "warning", "error", "in", "out"];

  declare baseClasses: string[];
  declare infoClasses: string[];
  declare warningClasses: string[];
  declare errorClasses: string[];
  declare inClasses: string[];
  declare outClasses: string[];

  initialize() {
    instance = this;
  }

  render(toast: Toast) {
    const id = `toast-${crypto.randomUUID()}`;
    const elem = document.createElement("div");
    const timeout = toast.timeout ?? DEFAULT_TIMEOUT;

    elem.id = id;
    elem.classList.add(...this.baseClasses);
    elem.classList.add(...this[`${toast.type}Classes`]);
    elem.classList.add(...this.inClasses);
    elem.innerText = toast.msg;

    this.element.appendChild(elem);

    if (timeout) {
      setTimeout(() => {
        elem.classList.remove(...this.inClasses);
        elem.classList.add(...this.outClasses);
        elem.addEventListener("animationend", () => elem.remove());
      }, timeout);
    }
  }
}

export function toast(toast: Toast) {
  if (!instance) throw new Error("no toasts controller");
  instance.render(toast);
}
