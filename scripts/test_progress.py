import copy
import unittest
import progress

class RegistryTests(unittest.TestCase):
    def setUp(self):self.data=progress.read('parity/features.yaml')
    def test_registry_is_schema_valid(self):progress.validate(self.data)
    def test_missing_field_rejected(self):
        del self.data['features'][0]['verification_status']
        with self.assertRaises(Exception):progress.validate(self.data)
    def test_unknown_dependency_rejected(self):
        self.data['features'][0]['dependencies']=['MISSING-999']
        with self.assertRaises(ValueError):progress.validate(self.data)
    def test_passing_claim_needs_run(self):
        self.data['features'][0]['verification_status']='passed'
        self.data['features'][0]['verification_runs']=[]
        with self.assertRaises(ValueError):progress.validate(self.data)
    def test_provisional_reference_never_establishes_parity(self):
        f=copy.deepcopy(self.data['features'][0]);f.update(implementation_status='implemented',verification_status='passed',reference_observation_status='observed',freshness='current')
        self.assertFalse(progress.parity_verified(f,{f['id']:f},{'frozen':False}))
