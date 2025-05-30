#!/usr/bin/env nu

# Get configuration for the invalid config script (intentionally fails)
def 'main config' [] {
  print "invalid yaml content"
}

# Process a resource (default handler)
def 'main reconcile' [] {
  let resource = $in
  print $"Processing: ($resource)"
}

# Main help function
def main [] {
  help main
}
