;;; todolog.el --- Browse todolog tasks in Emacs -*- lexical-binding: t; -*-

;; Copyright (C) 2026 Pablo Lozano

;; Author: Pablo Lozano
;; Maintainer: Pablo Lozano
;; Version: 0.1.0
;; Package-Requires: ((emacs "29.1"))
;; Keywords: tools, convenience
;; URL: https://github.com/lasantosr/todolog

;; This file is not part of GNU Emacs.

;;; Commentary:

;; This package provides a `tabulated-list-mode' interface for todolog tasks.
;; It shells out to the todolog CLI, so the `todolog' binary must be installed
;; and visible on `exec-path'.

;;; Code:

(require 'project)
(require 'tabulated-list)

(defgroup todolog nil
  "Browse todolog tasks."
  :group 'tools
  :prefix "todolog-")

(defcustom todolog-command "todolog"
  "Command used to run todolog."
  :type 'string
  :group 'todolog)

(defcustom todolog-buffer-name "*todolog tasks*"
  "Name of the todolog task list buffer."
  :type 'string
  :group 'todolog)

(defcustom todolog-preview-window-height 0.35
  "Height used for bottom preview windows.
When this is a float, it is interpreted as a fraction of the frame height."
  :type '(choice integer float)
  :group 'todolog)

(defface todolog-id-face
  '((t :inherit font-lock-constant-face))
  "Face used for todolog task IDs."
  :group 'todolog)

(defface todolog-location-face
  '((t :inherit font-lock-comment-face))
  "Face used for todolog file and line prefixes."
  :group 'todolog)

(defun todolog-project-root ()
  "Return the current project root, or `default-directory' outside projects."
  (if-let ((project (project-current)))
      (project-root project)
    default-directory))

(defun todolog--command-string (&rest args)
  "Return a shell command for `todolog-command' with ARGS."
  (mapconcat #'shell-quote-argument (cons todolog-command args) " "))

(defun todolog--parse-line (line)
  "Parse one todolog --emacs LINE into a plist."
  (when (string-match "\\`\\(.+\\):\\([0-9]+\\):[0-9]+: \\(.*\\) \\[\\([^][]+\\)\\]\\'" line)
    (list :file (match-string 1 line)
          :line (string-to-number (match-string 2 line))
          :text (match-string 3 line)
          :id (match-string 4 line))))

(defun todolog--read-open-tasks ()
  "Return open todolog tasks for the current project."
  (let ((output (shell-command-to-string
                 (todolog--command-string "list" "--open" "--emacs"))))
    (delq nil (mapcar #'todolog--parse-line (split-string output "\n" t)))))

(defun todolog--sort-by-location (left right)
  "Return non-nil when todolog entry LEFT appears before RIGHT by location."
  (let* ((left-task (car left))
         (right-task (car right))
         (left-file (plist-get left-task :file))
         (right-file (plist-get right-task :file)))
    (if (string= left-file right-file)
        (< (plist-get left-task :line)
           (plist-get right-task :line))
      (string< left-file right-file))))

(defun todolog--task-entry (task)
  "Return a `tabulated-list-mode' entry for TASK."
  (let ((id (plist-get task :id))
        (location (format "%s:%s"
                          (plist-get task :file)
                          (plist-get task :line)))
        (text (plist-get task :text)))
    (list task
          (vector (propertize id 'face 'todolog-id-face)
                  (concat (propertize location 'face 'todolog-location-face)
                          " "
                          text)))))

(defvar-keymap todolog-mode-map
  :doc "Keymap for `todolog-mode'."
  "RET" #'todolog-visit-task
  "g" #'todolog-refresh
  "s" #'todolog-scan
  "d" #'todolog-done-at-point
  "o" #'todolog-visit-task-other-window
  "r" #'todolog-open-at-point
  "v" #'todolog-preview-task
  "q" #'quit-window)

;;;###autoload
(define-derived-mode todolog-mode tabulated-list-mode "Todolog"
  "Major mode for browsing todolog tasks."
  (setq-local tabulated-list-format
              [("ID" 14 t)
               ("Task" 0 todolog--sort-by-location)])
  (setq-local tabulated-list-padding 0)
  (setq-local tabulated-list-sort-key '("Task" . nil))
  (setq-local truncate-lines t)
  (setq-local mode-line-process
              '("  RET open  o other  v preview  g refresh  d done  r reopen"))
  (tabulated-list-init-header))

;;;###autoload
(defun todolog-list-open ()
  "Show open todolog tasks in a dedicated buffer."
  (interactive)
  (let* ((default-directory (todolog-project-root))
         (buffer (get-buffer-create todolog-buffer-name)))
    (with-current-buffer buffer
      (todolog-mode)
      (setq-local default-directory default-directory)
      (todolog-refresh))
    (pop-to-buffer buffer)))

(defun todolog-current-task ()
  "Return the todolog task at point."
  (or (tabulated-list-get-id)
      (user-error "No todolog task on this line")))

(defun todolog--goto-task-line (line)
  "Move point to task LINE in the current buffer."
  (goto-char (point-min))
  (forward-line (1- line)))

(defun todolog--task-file (task)
  "Return the absolute file path for TASK."
  (expand-file-name (plist-get task :file) default-directory))

;;;###autoload
(defun todolog-refresh ()
  "Refresh the current todolog task buffer."
  (interactive)
  (setq tabulated-list-entries
        (mapcar #'todolog--task-entry (todolog--read-open-tasks)))
  (tabulated-list-print t)
  (message "Loaded %d open todolog task%s"
           (length tabulated-list-entries)
           (if (= (length tabulated-list-entries) 1) "" "s")))

;;;###autoload
(defun todolog-visit-task ()
  "Open the todolog task at point."
  (interactive)
  (let* ((task (todolog-current-task))
         (file (todolog--task-file task))
         (line (plist-get task :line)))
    (find-file file)
    (todolog--goto-task-line line)))

;;;###autoload
(defun todolog-visit-task-other-window ()
  "Open the todolog task at point in another window."
  (interactive)
  (let* ((task (todolog-current-task))
         (file (todolog--task-file task))
         (line (plist-get task :line)))
    (find-file-other-window file)
    (todolog--goto-task-line line)))

;;;###autoload
(defun todolog-preview-task ()
  "Preview the todolog task at point in a bottom window."
  (interactive)
  (let* ((task (todolog-current-task))
         (file (todolog--task-file task))
         (line (plist-get task :line))
         (buffer (find-file-noselect file))
         (window (display-buffer-in-side-window
                  buffer
                  `((side . bottom)
                    (slot . 0)
                    (window-height . ,todolog-preview-window-height)))))
    (with-current-buffer buffer
      (todolog--goto-task-line line))
    (set-window-point window (with-current-buffer buffer (point)))))

;;;###autoload
(defun todolog-scan ()
  "Scan the current project for TODO-style comments."
  (interactive)
  (let ((default-directory (todolog-project-root)))
    (compile (todolog--command-string "scan" "."))))

;;;###autoload
(defun todolog-done (id)
  "Mark todolog task ID as done."
  (interactive "sTask ID: ")
  (let ((default-directory (todolog-project-root)))
    (shell-command (todolog--command-string "done" id))))

;;;###autoload
(defun todolog-done-at-point ()
  "Mark the todolog task at point as done and refresh the task buffer."
  (interactive)
  (let ((id (plist-get (todolog-current-task) :id)))
    (todolog-done id)
    (todolog-refresh)))

;;;###autoload
(defun todolog-open (id)
  "Reopen todolog task ID."
  (interactive "sTask ID: ")
  (let ((default-directory (todolog-project-root)))
    (shell-command (todolog--command-string "open" id))))

;;;###autoload
(defun todolog-open-at-point ()
  "Reopen the todolog task at point and refresh the task buffer."
  (interactive)
  (let ((id (plist-get (todolog-current-task) :id)))
    (todolog-open id)
    (todolog-refresh)))

(provide 'todolog)

;;; todolog.el ends here
