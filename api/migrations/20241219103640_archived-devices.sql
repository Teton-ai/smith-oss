-- Add migration script here

ALTER TABLE public.device
ADD COLUMN archived boolean NOT NULL DEFAULT false;
