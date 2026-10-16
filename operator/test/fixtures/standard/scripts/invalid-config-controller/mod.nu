
# Get configuration for the invalid config controller (intentionally fails)
def 'main config' [] {
  print "invalid yaml: [missing brackets"
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
