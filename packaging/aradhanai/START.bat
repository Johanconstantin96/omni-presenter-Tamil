@echo off
rem Aradhanai portable: startet das Programm immer aus diesem Ordner,
rem damit Datenbank (omni-presenter.db) und Medien hier bleiben.
cd /d "%~dp0"
start "" "%~dp0omni-presenter.exe"
